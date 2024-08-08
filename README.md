zero2product study

应该创建你自己的配置文件 configuration.yaml



```yaml
application_port: 8000
database:
  host: your host
  port: your port
  username: your username
  password: your pass
  database_name: your db name
```



创建一个 .evn 文件给sqlx 使用

```
DATABASE_URL="postgres://username:password@host:port/dbname"
```

