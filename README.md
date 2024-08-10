# Building & Running

To build the OCI image and load it into a running OCI daemon:

```shell-session
bazel run //cimmeria:load
```

Then, it should be possible to run the image:

```
podman run -it --rm -e REPO_DB_URL=... -e STATIC_BASE_URL=... cimmeria:latest
```

# Development

CONAN_HOME is set by the `.envrc` if dotenv is installed. If dotenv is not
installed, this will need to be set explicitly. To initialize the Conan
settings, run the following:

```shell-session
conan profile detect
conan remote add localhost http://localhost:3000/
```

Then, we can install packages manually using a command like the following:

```
conan install -r=localhost --requires=stm32mp1/1.6.0
```

Exploring the Conan API can be done using combinations of `conan list` and
`curl`:

```
conan list -r=conancenter 'zlib/1.3.1#*'
curl https://center.conan.io/v2/conans/zlib/1.3.1/_/_/latest
```

For serving local static files, Python's built-in HTTP server does a fine job:

```shell-session
python -m http.server 8000
```
