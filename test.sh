# output: body + status code, read cookies from file
base=localhost:8080

description=""
expected=""
result=""

assert () {
  if [ "$result" != "$expected" ]
  then
    echo "Assertion failure: $description"
    echo "Expected:  '$expected'"
    echo "Actually:  '$result'"
    exit 1
  fi
}

# clear database
rm -f cookies
sqlite3 database.sqlite "delete from documents"
sqlite3 database.sqlite "delete from users"
sqlite3 database.sqlite "delete from user_sessions"

# Not logged in - all should fail
description="unauthorized get"
expected='unauthorized 401'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="unauthorized post"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base)
assert

description="unauthorized put"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base)
assert

description="unauthorized delete"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
assert

# Registered and logged in
description="register user"
expected="user created 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/register)
assert

description="logout"
expected="logged out 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert

description="login"
expected="logged in 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/login)
assert

description="get non-existent data"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="insert data"
expected="created 201"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base)
assert

description="insert data again"
expected="conflict 409"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base)
assert

description="get inserted data"
expected='{"id":"12345","data":{ "b" : 111 }} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="update inserted data"
expected="updated 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base)
assert

description="get updated data"
expected='{"id":"12345","data":{"c": 444}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="delete data"
expected="deleted 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
assert

description="delete missing data"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
assert

# insert data for cross-user access
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base)

description="logout"
expected="logged out 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert

# Registered another user
description="register thief user"
expected="user created 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=thief&password=1234' $base/user/register)
assert

description="get asdf's data logged in as thief"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="insert thief data"
expected="created 201"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/thief_data.json -H 'content-type: application/json' $base)
assert

description="get thief's inserted data"
expected='{"id":"12345","data":{ "thief": true }} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="update inserted data"
expected="updated 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base)
assert

description="get updated data"
expected='{"id":"12345","data":{"c": 444}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
assert

description="delete data"
expected="deleted 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
assert

description="logout"
expected="logged out 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert
