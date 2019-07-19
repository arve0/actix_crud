# output: body + status code, read cookies from file
base=localhost:8080

assert () {
  if [ "$1" != "$2" ]
  then
    echo "Assertion failure:  '$1' != '$2'"
    exit 1
  fi
}

# clear database
rm -f cookies
sqlite3 database.sqlite "delete from entries"
sqlite3 database.sqlite "delete from users"
sqlite3 database.sqlite "delete from user_sessions"

# register user
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/register)
expected="user created 200"
assert "$result" "$expected"

# logout
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
expected="logged out 200"
assert "$result" "$expected"

# login
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/login)
expected="logged in 200"
assert "$result" "$expected"

# insert data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X POST -d @src/test_data.json -H 'content-type: application/json' $base)
expected="created 201"
assert "$result" "$expected"

# get inserted data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
expected='{"id":"12345","data":{ "b" : 111 }} 200'
assert "$result" "$expected"

# update inserted data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X PUT -d @src/test_data_update.json -H 'content-type: application/json' $base)
expected="updated 200"
assert "$result" "$expected"

# get updated data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/12345)
expected='{"id":"12345","data":{"c": 444}} 200'
assert "$result" "$expected"

# delete data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
expected="deleted 200"
assert "$result" "$expected"

# delete missing data
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies -X DELETE $base/12345)
expected="not found 404"
assert "$result" "$expected"

# logout
result=$(curl -s -w ' %{http_code}' -b cookies -c cookies $base/user/logout)
expected="logged out 200"
assert "$result" "$expected"
