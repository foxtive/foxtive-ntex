# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

### 0.2.1
* fix(result): 'is_empty' should return false when value is present
* fix(responder): should return appropriate status code

### 0.2.0
* refactor(defs): renamed to 'ext'
* impl From<Error> for ResponseError
* HttpResult is now Result<ntex::web::HttpResponse, ResponseError>

