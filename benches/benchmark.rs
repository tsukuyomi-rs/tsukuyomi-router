#![feature(test)]
extern crate test;

use test::Bencher;

#[bench]
fn route_recognizer(b: &mut Bencher) {
    let mut router = route_recognizer::Router::new();
    router.add("/posts/:post_id/comments/:id", "comment");
    router.add("/posts/:post_id/comments", "comments");
    router.add("/posts/:post_id", "post");
    router.add("/posts", "posts");
    router.add("/comments", "comments2");
    router.add("/comments/:id", "comment2");

    b.iter(|| router.recognize("/posts/100/comments/200"));
}

#[bench]
fn tsukuyomi_router(b: &mut Bencher) -> tsukuyomi_router::Result<()> {
    let mut router = tsukuyomi_router::Router::new();
    router.add_route("/posts/:post_id/comments/:id", "comment")?;
    router.add_route("/posts/:post_id/comments", "comments")?;
    router.add_route("/posts/:post_id", "post")?;
    router.add_route("/posts", "posts")?;
    router.add_route("/comments", "comments2")?;
    router.add_route("/comments/:id", "comment2")?;

    let res = router.recognize("/posts/100/comments/200");
    if let Some((route, Some(params))) = res.route() {
        assert_eq!(*route.data(), "comment");
        assert_eq!(params.get(0), Some("100"));
        assert_eq!(params.get(1), Some("200"));
        assert!(params.get_wildcard().is_none());
    } else {
        panic!("unexpected condition");
    }

    b.iter(|| router.recognize("/posts/100/comments/200"));

    Ok(())
}
