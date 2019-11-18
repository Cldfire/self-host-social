<script>
    import { navigateTo } from 'svero';

    export let router = {};
</script>

<img alt="profile picture" src="/api/profile-pic/{router.params.userId}">
