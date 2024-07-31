# Development

CONAN_HOME is set by the `.envrc` if dotenv is installed. If dotenv is not
installed, this will need to be set explicitly. To set up the Conan settings,
run the following:

```shell-session
conan profile detect
conan remote add localhost http://localhost:3000/
```

Then, we can install packages manually using a command like the following:

```
conan install -r=localhost --requires=stm32mp1/1.6.0
```
