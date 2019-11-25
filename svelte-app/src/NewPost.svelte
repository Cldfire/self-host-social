<script>
    import { navigateTo } from 'svero';

    import { userId } from './stores.js';
    // TODO: replace with official get fn when that is available
    import { get_store_value } from 'svelte/internal'

    async function handleSubmit(event) {
        event.target.create.disabled = true;
        const user_page = "/user/" + get_store_value(userId);
        var creationResponse = {};

        var reader = new FileReader();

        // TODO: when browsers support it, move back to using the far cleaner
        // Blob.arrayBuffer()
        //
        // or get a polyfill working
        reader.addEventListener("loadend", async function () {
            const arrayBuf = reader.result;

            const imgResponse = await fetch(
                "/api/set-post-image/" + creationResponse.post_id,
                {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/octet-stream'
                    },
                    credentials: 'same-origin',
                    body: arrayBuf
                }
            );

            if (!imgResponse.ok) {
                alert("image upload failed");
            }

            navigateTo(user_page);
        }, false);

        if(!event.target.checkValidity()) {
            event.target.create.disabled = false;
            return;
        }

        const response = await fetch(
            "/api/create-post",
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'same-origin',
                body: JSON.stringify({
                    body: event.target.body.value
                })
            }
        );

        if (response.ok) {
            var img = event.target.file.files[0];

            if (img) {
                creationResponse = await response.json();
                reader.readAsArrayBuffer(img);
            } else {
                navigateTo(user_page);
            }
        } else {
            // TODO: handle potential errors / issues
            // should reply with json payload
            alert("creation failed");
            event.target.create.disabled = false;
        }
    }

    function previewFile() {
        var preview = document.getElementById("preview");
        var file = document.getElementById("file").files[0];
        var reader = new FileReader();

        reader.addEventListener("load", function () {
            preview.src = reader.result;
        }, false);

        if (file) {
            reader.readAsDataURL(file);
        }
    }
</script>

<form on:submit|preventDefault="{handleSubmit}">
    <label for="body">Post Description</label>
    <textarea id="body"></textarea>

    <label for="file">Choose Image</label>
    <input id="file" type="file" on:change="{previewFile}" accept="image/*">

    <button id="create" type="submit">Create post</button>
</form>

<img id="preview" src="" width="300" alt="Image preview...">
