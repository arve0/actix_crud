#!/usr/bin/env bash
# output: body + status code, read cookies from file
base=localhost:8080

description=""
expected=""
result=""

assert () {
  if [[ $result != *"$expected"* ]]
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

# Index
description="server is up"
expected=' 200'
result=$(curl -s -w ' %{http_code}' $base)
assert

# static folder
echo -n "asdf" > client/public/static/file.txt
description="serves static files"
expected='asdf 200'
result=$(curl -s -w ' %{http_code}' $base/static/file.txt)
assert
rm client/public/static/file.txt

# Not logged in - all should fail
description="unauthorized get"
expected='unauthorized 401'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/12345)
assert

description="unauthorized post"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base/document)
assert

description="unauthorized put"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base/document/12345)
assert

description="unauthorized delete"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/document/12345)
assert

# Registered and logged in
description="register user"
expected="user registered 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/register)
assert

description="logout"
expected="logged out 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert

description="login"
expected="logged in 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/login)
assert

description="get non-existent data"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/12345)
assert

description="insert data"
expected=" 201"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base/document)
assert
id_not_deleted=$(echo $result | sed -e 's/ 201//')

description="insert data again"
expected=" 201"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base/document)
assert
id=$(echo $result | sed -e 's/ 201//')

description="get inserted data"
expected='{"id":"'
expected+=$id
expected+='","data":{"b" : 111}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/$id)
assert

description="update inserted data"
expected="updated 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base/document/$id)
assert

description="get updated data"
expected='{"id":"'
expected+=$id
expected+='","data":{"c": 444}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/$id)
assert

description="delete data"
expected="deleted 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/document/$id)
assert

description="delete missing data"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/document/$id)
assert

# insert data for cross-user access
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/data.json -H 'content-type: application/json' $base)

description="logout"
expected="logged out 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert

# Registered another user
description="register thief user"
expected="user registered 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=thief&password=1234' $base/user/register)
assert

description="get asdf's data logged in as thief"
expected="not found 404"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/$id_not_deleted)
assert

description="insert thief data"
expected=" 201"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @test/thief_data.json -H 'content-type: application/json' $base/document)
assert
id=$(echo $result | sed -e 's/ 201//')

description="get thief's inserted data"
expected='{"id":"'
expected+=$id
expected+='","data":{"thief": true}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/$id)
assert

description="update inserted data"
expected="updated 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @test/data_update.json -H 'content-type: application/json' $base/document/$id)
assert

description="get updated data"
expected='{"id":"'
expected+=$id
expected+='","data":{"c": 444}} 200'
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/document/$id)
assert

description="delete data"
expected="deleted 200"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/document/$id)
assert

description="logout"
expected="logged out 303"
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
assert

echo "All tests OK"
