<script>
    import Post from './Post.svelte'

    import { loadRecentPosts } from './utils.js'

    export let router;
    let userInfoPromise = loadUserInfo();
    // TODO: load more than the last 10 posts
    let recentPostsPromise = loadRecentPosts(router.params.userId, 10);

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
</script>


{#await userInfoPromise then userInfo}
    <img alt="profile picture" src="/api/profile-pic/{router.params.userId}" height=100>

    <h3>{userInfo.displayName}</h3>
    <p>{userInfo.realName}</p>

    {#await recentPostsPromise then recentPosts}
        {#each recentPosts as post}
            <Post postInfo={post}/>
        {/each}
    {/await}
{/await}
