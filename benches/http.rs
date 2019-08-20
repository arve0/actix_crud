/**
 * What did I learn from this benchmark?
 *
 */
use actix_crud::{db, document, user};
use actix_session::CookieSession;
use actix_web::dev::Service;
use actix_web::{test, web, App};
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
}
