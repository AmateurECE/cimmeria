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
