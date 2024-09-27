CREATE TABLE payment (
    id SERIAL,
    transaction_id VARCHAR(100),
    request_id VARCHAR(100),
    currency VARCHAR(50),
    provider_name VARCHAR(100),
    amount INTEGER,
    payment_dt BIGINT,
    bank VARCHAR(100),
    delivery_cost INTEGER,
    goods_total INTEGER,
    custom_fee INTEGER
);


CREATE TABLE item (
    chrt_id SERIAL,
    track_number VARCHAR(100),
    price INTEGER,
    rid VARCHAR(100),
    item_name VARCHAR(200),
    sale INTEGER,
    item_size VARCHAR(100),
    total_prize INTEGER,
    nm_id BIGINT,
    brand VARCHAR(100),
    item_status INTEGER
);

CREATE TABLE delivery (
    id SERIAL,
    del_name VARCHAR(200),
    phone VARCHAR(20),
    zip VARCHAR(50),
    city VARCHAR(100),
    del_address VARCHAR(200),
    region VARCHAR(100),
    email VARCHAR(100)
);

CREATE TABLE order_to_item (
    id SERIAL,
    order_id INTEGER,
    item_id INTEGER
);

CREATE TABLE model (
    id SERIAL,
    order_uid VARCHAR(100),
    track_number VARCHAR(100),
    model_entry VARCHAR(100),
    delivery_id INTEGER,
    payment_id INTEGER,
    locale VARCHAR(10),
    internal_signature VARCHAR(100),
    customer_id VARCHAR(100),
    delivery_service VARCHAR(100),
    shardkey VARCHAR(50),
    sm_id BIGINT,
    date_created VARCHAR(100),
    oof_shard VARCHAR(50)
);