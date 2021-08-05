use super::*;

mod for_handlers {
    use super::*;

    #[tokio::test]
    async fn get_handles_head() {
        let app = route(
            "/",
            get(|| async {
                let mut headers = HeaderMap::new();
                headers.insert("x-some-header", "foobar".parse().unwrap());
                (headers, "you shouldn't see this")
            }),
        );

        let addr = run_in_background(app).await;

        let client = reqwest::Client::new();

        let res = client
            .head(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-some-header"], "foobar");
        assert_eq!(res.headers()["content-length"], "0");
        assert_eq!(res.text().await.unwrap(), "");
    }

    #[tokio::test]
    async fn head_explicitly_defined_last() {
        let app = route(
            "/",
            get(|| async { "hi from GET" }).head(|| async {
                let mut headers = HeaderMap::new();
                headers.insert("x-some-header", "foobar".parse().unwrap());
                (headers, "you shouldn't see this")
            }),
        );

        let addr = run_in_background(app).await;

        let client = reqwest::Client::new();

        let res = client
            .head(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-some-header"], "foobar");
        assert_eq!(res.headers()["content-length"], "0");
        assert_eq!(res.text().await.unwrap(), "");

        let res = client
            .get(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "hi from GET");
    }

    #[tokio::test]
    async fn head_explicitly_defined_first() {
        let app = route(
            "/",
            head(|| async {
                let mut headers = HeaderMap::new();
                headers.insert("x-some-header", "foobar".parse().unwrap());
                (headers, "you shouldn't see this")
            })
            .get(|| async { "hi from GET" }),
        );

        let addr = run_in_background(app).await;

        let client = reqwest::Client::new();

        let res = client
            .head(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-some-header"], "foobar");
        assert_eq!(res.headers()["content-length"], "0");
        assert_eq!(res.text().await.unwrap(), "");

        let res = client
            .get(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "hi from GET");
    }

    #[tokio::test]
    async fn head_explicitly_defined_middle() {
        let app = route(
            "/",
            post(|| async { "hi from POST" })
                .head(|| async {
                    let mut headers = HeaderMap::new();
                    headers.insert("x-some-header", "foobar".parse().unwrap());
                    (headers, "you shouldn't see this")
                })
                .get(|| async { "hi from GET" }),
        );

        let addr = run_in_background(app).await;

        let client = reqwest::Client::new();

        let res = client
            .head(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()["x-some-header"], "foobar");
        assert_eq!(res.headers()["content-length"], "0");
        assert_eq!(res.text().await.unwrap(), "");

        let res = client
            .get(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "hi from GET");

        let res = client
            .post(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "hi from POST");
    }
}

mod for_services {
    use super::*;
}
