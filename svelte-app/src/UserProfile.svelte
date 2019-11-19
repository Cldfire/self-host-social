<script>
    import { onMount } from 'svelte';
    import { navigateTo } from 'svero';

    export let router;
    let userInfoPromise = loadUserInfo();

    async function loadUserInfo() {
        const response = await fetch(
            "/api/user-info/" + router.params.userId,
            {
                method: 'GET',
                credentials: 'same-origin'
            }
        );
        const json = await response.json();

        if (response.ok) {
            return {
                realName: json.real_name,
                displayName: json.display_name
            };
        }
    }
</script>


{#await userInfoPromise then userInfo}
    <img alt="profile picture" src="/api/profile-pic/{router.params.userId}" height=100 width=100>

    <h3>{userInfo.displayName}</h3>
    <p>{userInfo.realName}</p>
{/await}
