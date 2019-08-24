<script>
    let user = fetch('/user').then(r => {
        if (r.status !== 200) {
            throw new Error(`Got ${r.statusText} (${r.status}), body: ${r.body}`);
        }
        return r.text();
    });

    let register = false;
    let login = false;

    function loginDialog() {
        register = false;
        login = true;
    }

    function registerDialog() {
        login = false;
        register = true;
    }
</script>

{#await user then username}
    <span>Logged in as {username}.</span> <a href="/user/logout">Logout</a>
{:catch}
    <a on:click={loginDialog}>Log in</a>
    <a on:click={registerDialog}>Register</a>

    {#if login}
        <form action="/user/login" method="POST">
            <label>Username <input name=username></label>
            <label>Password <input name=password type=password></label>
            <button type=submit>Login</button>
        </form>
    {/if}

    {#if register}
        <form action="/user/register" method="POST">
            <label>Username <input name=username></label>
            <label>Password <input name=password type=password></label>
            <button type=submit>Register</button>
        </form>
    {/if}
{/await}
