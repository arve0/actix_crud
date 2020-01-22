/**
 * What did I learn from this benchmark?
 * - Inserting with id was 10 times faster then without id, why?
 *      Was inserting the same document, not actually inserting
 *      a new one, by generating the same "random" id each time.
 *      Fixed by creating rng outside bench-iteration.
 */
use actix_crud;
use actix_web::dev::Service;
use actix_web::{http, test, App};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;

criterion_group!(benches, http_benchmark);
criterion_main!(benches);

const DB_FILENAME: &str = "storage/http-bench.sqlite";

fn http_benchmark(c: &mut Criterion) {
    c.bench_function("get index", |b| {
        let mut app = test::init_service(
            App::new().configure(|cfg| actix_crud::App::create(DB_FILENAME).config(cfg)),
        );

        b.iter(|| {
            let req = test::TestRequest::with_uri("/").to_request();
            let response = test::block_on(app.call(req)).unwrap();
            assert_eq!(response.status(), http::StatusCode::OK);
        })
    });

    c.bench_function("insert document", |b| {
        clear_db();

        let mut app = test::init_service(
            App::new().configure(|cfg| actix_crud::App::create(DB_FILENAME).config(cfg)),
        );

        let login_request = test::TestRequest::post()
            .uri("/user/register")
            .header("content-type", "application/x-www-form-urlencoded")
            .set_payload("username=a&password=1234")
            .to_request();
        let login_response = test::block_on(app.call(login_request)).unwrap();
        assert_eq!(login_response.status(), http::StatusCode::SEE_OTHER);

        let cookie = login_response
            .response()
            .cookies()
            .find(|c| c.name() == "actix-session")
            .unwrap();

        let document = r#"{"b":111}"#;

        b.iter(|| {
            let create_request = test::TestRequest::post()
                .uri("/document")
                .cookie(cookie.clone())
                .header("content-type", "application/json")
                .set_payload(document)
                .to_request();
            let create_response = test::block_on(app.call(create_request)).unwrap();
            assert_eq!(create_response.status(), http::StatusCode::CREATED);
        })
    });

    c.bench_function("insert document with random id", |b| {
        clear_db();

        let mut app = test::init_service(
            App::new().configure(|cfg| actix_crud::App::create(DB_FILENAME).config(cfg)),
        );

        let login_request = test::TestRequest::post()
            .uri("/user/register")
            .header("content-type", "application/x-www-form-urlencoded")
            .set_payload("username=a&password=1234")
            .to_request();
        let login_response = test::block_on(app.call(login_request)).unwrap();
        assert_eq!(login_response.status(), http::StatusCode::SEE_OTHER);

        let cookie = login_response
            .response()
            .cookies()
            .find(|c| c.name() == "actix-session")
            .unwrap();
        let mut rng = SmallRng::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 2, 3, 4, 5, 6, 7]);

        b.iter(|| {
            let id = get_random_id(&mut rng);
            let uri = ["/document/", &id.to_string()].concat();
            let document = ["[", &id.to_string(), "]"].concat();
            let create_request = test::TestRequest::post()
                .uri(&uri)
                .cookie(cookie.clone())
                .header("content-type", "application/json")
                .set_payload(document)
                .to_request();
            let create_response = test::block_on(app.call(create_request)).unwrap();
            assert_eq!(create_response.status(), http::StatusCode::CREATED);
        })
    });

    c.bench_function("get document", |b| {
        clear_db();

        let mut app = test::init_service(
            App::new().configure(|cfg| actix_crud::App::create(DB_FILENAME).config(cfg)),
        );

        let login_request = test::TestRequest::post()
            .uri("/user/register")
            .header("content-type", "application/x-www-form-urlencoded")
            .set_payload("username=a&password=1234")
            .to_request();
        let login_response = test::block_on(app.call(login_request)).unwrap();
        assert_eq!(login_response.status(), http::StatusCode::SEE_OTHER);

        let cookie = login_response
            .response()
            .cookies()
            .find(|c| c.name() == "actix-session")
            .unwrap();

        let document = r#"{"b":111}"#;
        let uri = "/document/123";
        let create_request = test::TestRequest::post()
            .uri(uri)
            .cookie(cookie.clone())
            .header("content-type", "application/json")
            .set_payload(document)
            .to_request();
        let create_response = test::block_on(app.call(create_request)).unwrap();
        assert_eq!(create_response.status(), http::StatusCode::CREATED);

        let saved_document = test::read_body(create_response);

        let request = test::TestRequest::with_uri(uri)
            .cookie(cookie.clone())
            .to_request();
        let response = test::read_response(&mut app, request);
        assert_eq!(response, saved_document);

        b.iter(|| {
            let request = test::TestRequest::with_uri(uri)
                .cookie(cookie.clone())
                .to_request();
            let response = test::read_response(&mut app, request);
            assert_eq!(response, saved_document);
        })
    });
}

fn clear_db() {
    std::fs::remove_file(DB_FILENAME).unwrap_or(());
}

fn get_random_id(rng: &mut SmallRng) -> u32 {
    rng.gen_range(0, u32::max_value())
}
