<script>
    import InputForm from './InputForm.svelte';
    import { documents } from './document.service.js';

    let show_input_form = false
    let document = {}

    function create() {
        document = {
            id: null,
            data: {
                date: "",
                description: "",
                amount: 0,
                done: false,
            }
        }
        show_input_form = true
    }
    function edit(doc) {
        document = doc
        show_input_form = true
    }
    function done() {
        show_input_form = false
        document = {}
    }
    function delete_(id) {
        if (document.id === id) {
            show_input_form = false
            document = {}
        }
        documents.delete_(id)
    }
</script>

<div class="hero">
    <div class="hero-body">
        <h1 class="title">Documents</h1>
{#if $documents.length}
        <table class=table>
            <thead>
                <tr>
                    <!-- <th>id</th> -->
                    <th>date</th>
                    <th>description</th>
                    <th>amount</th>
                    <th>done</th>
                    <th>actions</th>
                </tr>
            </thead>
        {#each $documents as doc (doc.id)}
            <tr>
                <!-- <td>{doc.id}</td> -->
                <td>{doc.data.date}</td>
                <td>{doc.data.description}</td>
                <td>{doc.data.amount}</td>
                <td>{doc.data.done ? "✅" : "❌"}</td>
                <td>
                    <button on:click={() => edit(doc)}>Edit</button>
                    <button on:click={() => delete_(doc.id)}>Delete</button>
                </td>
            </tr>
        {/each}
        </table>
        {/if}
        {#if show_input_form}
        <InputForm
        document={document}
        on:done={done}
        />
        {:else}
        <button on:click={create}>Create new document</button>
{/if}
    </div>
</div>
