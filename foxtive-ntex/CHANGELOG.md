# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

### 0.2.0
* refactor(defs): renamed to 'ext'
* impl From<Error> for ResponseError
* HttpResult is now Result<ntex::web::HttpResponse, ResponseError>

