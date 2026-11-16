export type Post = {
  id: number;
  user: {
    id: number;
    username: string;
  };
  text: string | null;
  image: string;
};

// build one post "card"
export function makePostShared(
  post: Post,
  currentUserId: number | null,
  deleteCallback: () => Promise<void>,
) {
  const card = document.createElement("article");
  card.className = "post";
  const username = document.createElement("a");
  username.href = `/user.html?user=${post.user.id}`;
  username.className = "post-username";
  username.textContent = post.user.username;
  card.append(username);

  if (post.text) {
    const text = document.createElement("p");
    text.className = "post-text";
    text.textContent = post.text;
    card.append(text);
  }

  const img = document.createElement("img");
  img.className = "post-image";
  img.src = post.image;
  img.loading = "lazy";
  img.onerror = () => card.classList.add("post-image-missing");

  card.append(img);

  if (currentUserId !== null && post.user.id === currentUserId) {
    const deleteButton = document.createElement("button");
    deleteButton.type = "button";
    deleteButton.className = "post-delete";
    deleteButton.textContent = "Delete post";

    deleteButton.addEventListener("click", async () => {
      if (!window.confirm("Delete this post?")) {
        return;
      }

      deleteButton.disabled = true;

      try {
        const response = await fetch("/api/post/delete/" + post.id, {
          method: "POST",
          credentials: "include",
        });

        if (!response.ok) {
          throw new Error("Delete failed: " + response.status);
        }

        await deleteCallback();
      } catch (error) {
        console.error(error);
        window.alert("Could not delete the post. Please try again.");
        deleteButton.disabled = false;
      }
    });

    card.append(deleteButton);
  }

  return card;
}
