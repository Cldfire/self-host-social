<script>
    // Gives us the post id, the post body, when the post was created, and the user id
    // for the user that made the post
    export let postInfo;
</script>

<p>{postInfo.body}</p>
<img alt="post {postInfo.id} img" src="/api/post-image/{postInfo.id}" width="300">
