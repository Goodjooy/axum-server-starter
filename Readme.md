# Server Starter

## 背景

随着服务器的功能逐步丰富，启动时需要准备的内容也极大地扩大了，这也使得 main 函数里面出现了异常复杂的启动流程。
复杂的启动流程会使得 main 里面的代码过于复杂，且可能出现难以被注意到的错误。为了进一步简化启动区的代码，提供简洁的服务启动过程代码
因此制作 Server Starter 以达到目的

## 使用

注意：**这只是预期的使用效果**

```rust
#[tokio::main]
async fn main(){
    let config = load_config();


    serve::with_config(config)
    // things need initial
    .append(TracingLogger)
    .append(Mysql)
    .append(MongoDb)
    .append(Redis)
    .append(BiliClient)
    .append(QiniuClient)
    .append(DunTaskManager)
    // register the router
    .append(Router)
    // adding middleware
    .with_global_middleware(Middleware)
    // running all pending task, ready for launch
    .prepare_serve_start()
    .await
    .expect("Prepare Starting Server Error")
    // launch
    .launch()
    .await
    .expect("Server Error")
}
```
