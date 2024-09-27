use std::error::Error;
use tokio_postgres::Client;
use crate::models::{Payment, Delivery, Order, Item};
use log::info;


// Connection structure which can be used like class to connect the database
pub struct Connection {
    client: Client,
}

impl Connection {
    // Insert a payment into database
    async fn insert_into_payment(&self, payment: &Payment) -> Result<i32, Box<dyn Error>> {
        info!("Starting inserting the payment {}", payment.transaction);
    
        let row = self.client.query_one(
            "INSERT INTO payment (
                transaction_id,
                request_id,
                currency,
                provider_name,
                amount,
                payment_dt,
                bank,
                delivery_cost,
                goods_total,
                custom_fee
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id;", 
             &[
                &payment.transaction, &payment.request_id, &payment.currency, 
                &payment.provider, &payment.amount, &payment.payment_dt,
                &payment.bank, &payment.delivery_cost, &payment.goods_total,
                &payment.custom_fee
             ]
        ).await?;

        let payment_id: i32 = row.get(0);
    
        info!("Done inserting the payment {}", payment.transaction);
    
        Ok(payment_id)
    }

    // Inserting a delivery into database
    async fn insert_into_delivery(&self, delivery: &Delivery) -> Result<i32, Box<dyn Error>> {
        info!("Starting inserting the delivery {}", delivery.name);

        let row = self.client.query_one(
            "INSERT INTO delivery (
                del_name,
                phone,
                zip,
                city,
                del_address,
                region,
                email
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id;", 
             &[
                &delivery.name, &delivery.phone, &delivery.zip,
                &delivery.city, &delivery.address, &delivery.region,
                &delivery.email
             ]
        ).await?;

        let delivery_id: i32 = row.get(0);

        info!("Done the delivery {}", delivery.name);

        Ok(delivery_id)
    }

    // Inserting an item into delivery   
    async fn insert_into_item(&self, item: &Item) -> Result<i32, Box<dyn Error>> {
        info!("Starting inserting the item {}", item.chrt_id);

        let row = self.client.query_one(
            "INSERT INTO item (
                chrt_id,
                track_number,
                price,
                rid,
                item_name,
                sale,
                item_size,
                total_prize,
                nm_id,
                brand,
                item_status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING chrt_id;", 
             &[
                &item.chrt_id, &item.track_number, &item.price,
                &item.rid, &item.name, &item.sale, &item.size,
                &item.total_price, &item.nm_id, &item.brand,
                &item.status
             ]
        ).await?;

        let item_id: i32 = row.get(0);

        info!("Done inserting the item {}", item.chrt_id);

        Ok(item_id)
    }

    // Inserting order_id and item_chrt_id into temporary table which is used for making many_to_many connection between tables model and item
    async fn insert_into_order_to_item(&self, order_id: i32, item_id: i32) -> Result<(), Box<dyn Error>> {
        info!("Starting inserting the order {} and item {} into order_to_item", order_id, item_id);

        self.client.execute(
            "INSERT INTO order_to_item (
                order_id,
                item_id
            ) VALUES ($1, $2);", 
             &[&order_id, &item_id]
        ).await?;

        info!("Done inserting the order {} and item {} into order_to_item", order_id, item_id);

        Ok(())
    }

    // Inserting order into database
    pub async fn insert_into_order(&self, order: &Order) -> Result<(), Box<dyn Error>> {
        info!("Starting inserting the order {}", order.order_uid);

        let delivery_id = self.insert_into_delivery(&order.delivery).await?;
        let payment_id = self.insert_into_payment(&order.payment).await?;
        info!("strart Order");
        let row = self.client.query_one(
            "INSERT INTO model (
                order_uid,
                track_number,
                model_entry,
                delivery_id,
                payment_id,
                locale,
                internal_signature,
                customer_id,
                delivery_service,
                shardkey,
                sm_id,
                date_created,
                oof_shard
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id;",
             &[
                &order.order_uid, &order.track_number, &order.entry,
                &delivery_id, &payment_id, &order.locale,
                &order.internal_signature, &order.customer_id, &order.delivery_service,
                &order.shardkey, &order.sm_id, &order.date_created, &order.oof_shard
             ]
        ).await?;

        let order_id: i32 = row.get(0);
        
        for item in &order.items {
            let item_id = self.insert_into_item(&item).await?;
            self.insert_into_order_to_item(order_id, item_id).await?;
        }

        info!("Done inserting order {}", order.order_uid);

        Ok(())
    }

    // Getting vector of all orders into the database
    pub async fn get_all_orders(&self) -> Result<Vec<Order>, Box<dyn Error>> {

        let rows = self.client.query("
        SELECT model.id, model.order_uid, model.track_number, model.model_entry,
            model.locale, model.internal_signature, model.customer_id, model.delivery_service, model.shardkey,
            model.sm_id, model.date_created, model.oof_shard, payment.transaction_id, payment.request_id, payment.currency,
            payment.provider_name, payment.amount, payment.payment_dt, payment.bank, payment.delivery_cost, payment.goods_total,
            payment.custom_fee, delivery.del_name, delivery.phone, delivery.zip, delivery.city, delivery.del_address,
            delivery.region, delivery.email
        FROM model
        JOIN delivery ON delivery.id = model.delivery_id
        JOIN payment ON payment.id = model.payment_id;
        ", &[]).await?;

        let mut v: Vec<Order> = Vec::new();

        for row in rows{
            let payment = self.make_payment_struct(&row);
            let delivery = self.make_delivery_struct(&row);

            let mut order = self.make_order_struct(&row, delivery, payment);

            order.items = self.get_items_for_order(row.get("id")).await?;
            v.push(order);
        }

        Ok(v)
    }

    // Make Payment structure from Row of tokio_postgres
    fn make_payment_struct(&self, row: &tokio_postgres::Row) -> Payment {
        Payment {
            transaction: row.get("transaction_id"),
            request_id: row.get("request_id"),
            currency: row.get("currency"),
            provider: row.get("provider_name"),
            amount: row.get("amount"),
            payment_dt: row.get("payment_dt"),
            bank: row.get("bank"),
            delivery_cost: row.get("delivery_cost"),
            goods_total: row.get("goods_total"),
            custom_fee: row.get("custom_fee"),
        }
    }

    // Make Delivery structure from Row of tokio_postgres
    fn make_delivery_struct(&self, row: &tokio_postgres::Row) -> Delivery {
        Delivery {
            name: row.get("del_name"),
            phone: row.get("phone"),
            zip: row.get("zip"),
            city: row.get("city"),
            address: row.get("del_address"),
            region: row.get("region"),
            email: row.get("email"),
        }
    }

    // Make Order structure from Row of tokio_postgres
    fn make_order_struct(&self, row: &tokio_postgres::Row, delivery: Delivery, payment: Payment) -> Order {
        Order {
            order_uid: row.get("order_uid"),
            track_number: row.get("track_number"),
            entry: row.get("model_entry"),
            delivery: delivery,
            payment: payment,
            items: vec![],
            locale: row.get("locale"),
            internal_signature: row.get("internal_signature"),
            customer_id: row.get("customer_id"),
            delivery_service: row.get("delivery_service"),
            shardkey: row.get("shardkey"),
            sm_id: row.get("sm_id"),
            date_created: row.get("date_created"),
            oof_shard: row.get("oof_shard"),
        }
    }

    // Getting vector of all items for one order into the database
    async fn get_items_for_order(&self, model_id: i32) -> Result<Vec<Item>, Box<dyn Error>> {
        let rows = self.client.query("
            SELECT item.chrt_id, item.track_number, item.price, item.rid,
                item.item_name, item.sale, item.item_size, item.total_prize,
                item.nm_id, item.brand, item.item_status
            FROM item
            INNER JOIN order_to_item ON item.chrt_id = order_to_item.item_id
            INNER JOIN model ON model.id = order_to_item.order_id
            WHERE model.id = $1;
        ", &[&model_id]).await?;

        let mut v: Vec<Item> = vec![];

        for row in rows {
            v.push(self.make_item_struct(&row));
        }

        Ok(v)
    }

    // Make Item structure from Row of tokio_postgres
    fn make_item_struct(&self, row: &tokio_postgres::Row) -> Item {
        Item {
            chrt_id: row.get("chrt_id"),
            track_number: row.get("track_number"),
            price: row.get("price"),
            rid: row.get("rid"),
            name: row.get("item_name"),
            sale: row.get("sale"),
            size: row.get("item_size"),
            total_price: row.get("total_prize"),
            nm_id: row.get("nm_id"),
            brand: row.get("brand"),
            status: row.get("item_status"),
        }
    }
}


// Make new connection class
pub fn new(client: Client) -> Connection{
    Connection {client: client}
}
