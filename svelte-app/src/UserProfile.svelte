<script>
    import Post from './Post.svelte'

    export let router;
    let userInfoPromise = loadUserInfo();
    let recentPostsPromise = loadRecentPosts();

    async function loadUserInfo() {
        const response = await fetch(
            "/api/user-info/" + router.params.userId,
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            const json = await response.json();

            return {
                realName: json.real_name,
                displayName: json.display_name
            };
        }
    }

    async function loadRecentPosts() {
        const response = await fetch(
            // TODO: load more than the last 10 posts
            "/api/recent-posts/" + router.params.userId + "/10",
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );

        if (response.ok) {
            return response.json();
        }
    }
</script>


{#await userInfoPromise then userInfo}
    <img alt="profile picture" src="/api/profile-pic/{router.params.userId}" height=100 width=100>

    <h3>{userInfo.displayName}</h3>
    <p>{userInfo.realName}</p>

    {#await recentPostsPromise then recentPosts}
        {#each recentPosts as post}
            <Post postInfo={post}/>
        {/each}
    {/await}
{/await}
