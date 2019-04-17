use tsukuyomi_router::Router;

#[test]
fn simple() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/domain/mime", "mime")?;
    router.add_route("/domain/yours", "yours")?;

    assert_eq!(
        router.recognize("/domain/mime").route().map(|r| r.data()),
        Some(&"mime")
    );
    assert_eq!(
        router.recognize("/domain/yours").route().map(|r| r.data()),
        Some(&"yours")
    );

    assert!(router.recognize("/domain/me").route().is_none());

    Ok(())
}

#[test]
fn param() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/posts/:post", "the_post")?;
    router.add_route("/posts/create", "create_post")?;

    assert_eq!(
        router.recognize("/posts/create").route().map(|r| r.data()),
        Some(&"create_post")
    );

    let res = router.recognize("/posts/12");
    assert_eq!(res.route().map(|r| r.data()), Some(&"the_post"));
    assert_eq!(&res.params().unwrap()[0], "12");

    Ok(())
}

#[test]
fn wildcard() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/public/*/index.html", "public_index")?;
    router.add_route("/*", "catch_all")?;

    {
        let res = router.recognize("/public/path/to/index.html");
        assert_eq!(res.route().map(|r| r.data()), Some(&"public_index"));
        assert_eq!(res.params().unwrap().get_wildcard(), Some("path/to"));
    }

    {
        let res = router.recognize("/path/to/index.html");
        assert_eq!(res.route().map(|r| r.data()), Some(&"catch_all"));
        assert_eq!(
            res.params().unwrap().get_wildcard(),
            Some("path/to/index.html")
        );
    }

    Ok(())
}
