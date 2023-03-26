# ProductivityDNS
A blacklist DNS to block all the addictive website to integrate with  Pomodoro App.

Need to use the `config.yaml` for the config: 
```rust 
 let config: Config = serde_yaml::from_str(include_str!("../config.yaml")).unwrap();
 ```
 
 
 
 To make testing purpose a simple `curl` with `POST`: 

 ```bash
 curl -d 'status=off' http://127.0.0.1:8000/status/ 
 ```
 The server use `rocket` to listen on the port 8000 when the POST request with `on` or `off` this will disable/enable the DNS to blacklist the name present into the `config.yaml` 
 
 
 ```yaml
 ---
bind: 0.0.0.0:53
domains:
  twitter.com:
    - name: '@'
      records: [127.0.0.1]
    - name: www
      records: [127.0.0.2]
    - name: '*'
      records: ['www']
      type: CNAME
  tyr1.test:
    - name: '@'
      records: [127.0.0.1]
    - name: 'abc'
      records: [127.0.0.1]
 ```
 
 ---
 
 ```rust
 #[post("/", data = "<status>")]
async fn status(status: &str, sender: &State<SenderPlease>) -> String {
    if status.contains("on") {
        info!("Status is {}", status);
        let mut sender = sender.inner().0.lock().await;
        if sender.is_none() {
            let (tx, rx) = mpsc::channel(1);
            run_thread_dns(rx);
            sender.insert(ShutdownSender(tx));
        }
    } else if status.contains("off") {
        let mut sender = sender.inner().0.lock().await;
        if let Some(tx) = sender.take() {
            tx.0.send(()).await;
        }
    } else {
        info!("Not suppose to be here is {}", status);
    }
    status.to_string()
}
```
