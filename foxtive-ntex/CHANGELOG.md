# Foxtive-Ntex Changelog
Foxtive-Ntex changelog file 

------

### 0.5.3 (2024-03-27)
* fix(http-error): convert and return proper error response

### 0.5.2 (2024-03-22)
* fix(message): return json-formatted error message

### 0.5.1 (2024-03-17)
* bump(foxtive): to fix app message issue

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

