<script>
    import Post from './Post.svelte';

    let searchResults = [];

    async function handleSubmit(event) {
        const response = await fetch(
            "/api/search-posts?query_string=" + encodeURI(event.target.query.value),
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            searchResults = await response.json();
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("search failed");
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="query">Search Query</label>
    <input required id="query"/>

    <button type="submit">Search</button>
</form>

<h2>Search Results</h2>

{#each searchResults as post}
    <Post postInfo={post}/>
{/each}
