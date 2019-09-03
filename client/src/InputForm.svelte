<script>
    import { documents } from './document.service.js';
    import { createEventDispatcher } from 'svelte'

    export let document;
    $: data = document.data;
    $: isNew = document.id === null;
    $: title = isNew ? "New" : "Edit";

    const dispatch = createEventDispatcher()

    function submit(event) {
        event.preventDefault()

        if (isNew) {
            documents.create(data).then(() => dispatch("done"))
        } else {
            documents.put(document).then(() => dispatch("done"))
        }
    }
</script>

<h2 class="subtitle">{title} document</h2>
<form on:submit={submit}>
    <label for=date>Date</label>
    <input id=date type=date bind:value={data.date}>

    <label for=description>Description</label>
    <input id=description type=text bind:value={data.description}>

    <label for=amount>Amount</label>
    <input id=amount type=number bind:value={data.amount}>

    <label for=done>Done</label>
    <input id=done type=checkbox bind:checked={data.done}>

    <div class=action>
        <button type=submit>Save</button>
        <button on:click={() => dispatch("done")}>Cancel</button>
    </div>
</form>

<style>
    label {
        display: block;
    }
    .action {
        margin-top: 1.5em;
    }
</style>
