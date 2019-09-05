<script>
    export let type = 'login';
    let prettyType = type[0].toUpperCase() + type.slice(1);
</script>

<h1>{prettyType}</h1>
<form action="/user/{type}" method="POST">

    <label for=username>Username</label>
    <input id=username name=username>

    <label for=password>Password</label>
    <input id=password name=password type=password>

    <button type=submit class=button>{prettyType}</button>
</form>

<style>
    label, button {
        display: block;
    }
</style>
