# Snake project

Currently, server application with api endpoints providing simple auth for login and session,
sending information about uploaded file, uploading file, getting information
about currently stored files and user information.

## Running
* Server require database running on ``127.0.0.1:8000`` to start
## Technologies used
* back-end framework: [Actix web](https://actix.rs)
* database: [SurrealDB](https://surrealdb.com)
* For all dependencies [see Cargo.toml](https://github.com/Yourzo/snake_project/blob/master/Cargo.toml)

## BIG plans for future:
* Finish backend so it's capable of planned functionality
* Create CLI client application (as a homework for c# class)
* Add possibility to control server remotely, even though there is not much to control
* Create client with user-friendly UI that could people like my sister or brother use
* Think and research some ways to check if requests from client didn't come for too long

## Small plans / TODOs:
* Decide if session id will be sent in url or in JSon data in body of request and standardize it along the code
* Send tree structure of directories that user can access
* Add session validation where ever needs to be
* Add session removing
* Add password encryption for user, figure out way to create master key
# ENDPOINTS:
Testing is done with postman
## List of tested endpoints:
* ``GET /all_allowed_files/{session_id}``
## List of endpoints to test:

## List of endpoints to finish:
Most of these are kinda working but some of them are missing additional functionality,
like session validation or password encryption.
* ``POST /file_upload_data/{session_id}``
* ``POST /upload_file/{session_id}``
* ``POST /login``
* ``GET /get_all_files_info/{session_id}``
* ``POST /set_user/{master_key}``
* ``GET /get_file/{session_id}``
## Planed endpoints:
Here I will add as I come up with what client app could need.
* ``GET /file_info/{session_id}``

## Current problems/ideas/thoughts
* There is problem with postman where I don't know how to send file and JSON in one http request
so actix can sort them properly,
if I could solve that I want to make ``file_upload_data`` and ``upload_file`` one function.
* Another idea is that I have to sort file: api/api.rs into few different file by what are they doing 