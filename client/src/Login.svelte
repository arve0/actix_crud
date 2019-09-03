<script>
    export let type = 'login';
    let prettyType = type[0].toUpperCase() + type.slice(1);
</script>

<div class="hero">
    <div class="hero-body">
        <form action="/user/{type}" method="POST">
            <h1 class="title">{prettyType}</h1>

            <label for=username>Username</label>
            <input id=username name=username>

            <label for=password>Password</label>
            <input id=password name=password type=password>

            <button type=submit class=button>{prettyType}</button>
        </form>
    </div>
</div>

<style>
    label, button {
        display: block;
    }
</style>
