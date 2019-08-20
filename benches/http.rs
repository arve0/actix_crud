/**
 * What did I learn from this benchmark?
 *
 */
use actix_crud::{db, document, user};
use actix_session::CookieSession;
use actix_web::dev::Service;
use actix_web::{http, test, web, App};
use actix_web::web::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, http_benchmark);
criterion_main!(benches);

fn http_benchmark(c: &mut Criterion) {
    c.bench_function("get index", |b| {
        let mut app = test::init_service(
            App::new()
                .data(db::get_pool())
                .wrap(CookieSession::signed(&[0; 32]).secure(false))
                .route("/", web::get().to(|| ""))
                .configure(user::config)
                .configure(document::config),
        );

        b.iter(|| {
            let req = test::TestRequest::with_uri("/test").to_request();
            test::block_on(app.call(req)).unwrap();
        })
    });

    c.bench_function("insert document", |b| {
        clear_db();

        let mut app = test::init_service(
            App::new()
                .data(db::get_pool())
                .wrap(CookieSession::signed(&[0; 32]).secure(false))
                .route("/", web::get().to(|| ""))
                .configure(user::config)
                .configure(document::config),
        );

        let login_request = test::TestRequest::post()
            .uri("/user/register")
            .header("content-type", "application/x-www-form-urlencoded")
            .set_payload("username=a&password=1234")
            .to_request();
        let login_response = test::block_on(app.call(login_request)).unwrap();
        let cookie = login_response.response()
            .cookies()
            .find(|c| c.name() == "actix-session")
            .unwrap();
        assert!(login_response.status() == http::StatusCode::CREATED);

        let mut i = 0;

        b.iter(|| {
            i += 1;
            let document = format!(r#"{{"id":"{}","data":{{"b":111}}}}"#, i);
            let create_request = test::TestRequest::post()
                .cookie(cookie.clone())
                .header("content-type", "application/json")
                .set_payload(document)
                .to_request();
            let create_response = test::block_on(app.call(create_request)).unwrap();
            assert!(create_response.status() == http::StatusCode::CREATED);
        })
    });

    c.bench_function("get document", |b| {
        clear_db();

        let mut app = test::init_service(
            App::new()
                .data(db::get_pool())
                .wrap(CookieSession::signed(&[0; 32]).secure(false))
                .route("/", web::get().to(|| ""))
                .configure(user::config)
                .configure(document::config),
        );

        let login_request = test::TestRequest::post()
            .uri("/user/register")
            .header("content-type", "application/x-www-form-urlencoded")
            .set_payload("username=a&password=1234")
            .to_request();
        let login_response = test::block_on(app.call(login_request)).unwrap();
        let cookie = login_response.response()
            .cookies()
            .find(|c| c.name() == "actix-session")
            .unwrap();
        assert!(login_response.status() == http::StatusCode::CREATED);

        let document = r#"{"id":"1","data":{"b":111}}"#;
        let create_request = test::TestRequest::post()
            .cookie(cookie.clone())
            .header("content-type", "application/json")
            .set_payload(document)
            .to_request();
        let create_response = test::block_on(app.call(create_request)).unwrap();
        assert!(create_response.status() == http::StatusCode::CREATED);

        let request = test::TestRequest::with_uri("/1")
            .cookie(cookie.clone())
            .to_request();
        let response = test::read_response(&mut app, request);
        let document = Bytes::from_static(document.as_bytes());
        assert!(response == document);

        b.iter(|| {
            let request = test::TestRequest::with_uri("/1")
                .cookie(cookie.clone())
                .to_request();
            let response = test::read_response(&mut app, request);
            assert!(response == document);
        })
    });
}

fn clear_db() {
    std::fs::remove_file("database.sqlite").unwrap_or(());
}
