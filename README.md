zero2product study

##### 应该创建你自己的配置文件 configuration

```
mkdir configuration
cd configuration
touch base.yaml

application:
  port: 8000
database:
  host: your host
  port: your port
  username: your username
  password: your pass
  database_name: your db name

touch local.yaml

application:
  host: 127.0.0.1
  
touch dev.yaml

application:
  host: 127.0.0.1
  
touch production.yaml

application:
  host: 0.0.0.0
  
```

##### 创建一个 .evn 文件给sqlx 使用

```
DATABASE_URL="postgres://username:password@host:port/dbname"
```

##### 根据 `Dockerfile` 中指定的配方构建标记为“zero2prod”的 docker 镜像

```
docker build --tag zero2prod --file Dockerfile .
docker run -p 8000:8000 zero2prod

curl -v http://127.0.0.1:8000/health_check


```

##### 可以创建 .dockerignore 来忽略下面的文件
```
.env
target/
tests/
Dockerfile
scripts/
migrations/
```