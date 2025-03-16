# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

------

### 0.5.0 (2024-03-16)
* refactor(features): remove unused
* feat(error): impl from validator::ValidationErrors & foxtive_ntex_multipart::MultipartError

### 0.4.0 (2024-03-16)
* fix: renamed 'FOXTIVE_WEB' to 'FOXTIVE_NTEX'

### 0.3.0
* feat(http): introduced HttpError to handle crate-level error

### 0.2.1
* fix(result): 'is_empty' should return false when value is present
* fix(responder): should return appropriate status code

### 0.2.0
* refactor(defs): renamed to 'ext'
* impl From<Error> for ResponseError
* HttpResult is now Result<ntex::web::HttpResponse, ResponseError>

