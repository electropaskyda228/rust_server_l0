# rust order server

## Отправка запросов
Отправка `json`-файлов осуществляется по адресу `http://host:port/add`. Перейдите по адресу `http://host:port/get`, чтобы получить `json`-файлы bp БД. `host` и `port` вы выбираете сами, когда запускаете сервер.

## Запуск сервера
Для запуска сервера необходимо ввести команду в папке проекта
```bash
cargo run -- --server-host localhost --server-port 8081 --db-user postgres --db-password George0404 --db-name rust_l0 --db-host localhost --db-port 5432
```
