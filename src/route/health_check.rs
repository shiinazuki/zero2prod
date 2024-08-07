use actix_web::HttpResponse;

// 我们一开始就返回了 `impl Responder`。
// 鉴于我们已经
// 更加熟悉 `actix-web`，我们现在明确说明类型。
// 性能没有差异！只是一种风格选择
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
