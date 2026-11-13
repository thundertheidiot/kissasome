import "./styles/index.css";

const authAction = document.querySelector<HTMLButtonElement>("#auth-action")!;
const logOutButton = document.querySelector<HTMLButtonElement>("#log-out")!;
const feed = document.querySelector<HTMLElement>("#feed")!;
const postForm = document.querySelector<HTMLFormElement>("#post-form")!;
const postImage = document.querySelector<HTMLInputElement>("#post-image")!;
const postText = document.querySelector<HTMLTextAreaElement>("#post-text")!;
const postError = document.querySelector<HTMLParagraphElement>("#post-error")!;
const postCancel = document.querySelector<HTMLButtonElement>("#post-cancel")!;

let currentUserId: number | null = null;

type Post = {
  id: number;
  user: {
    id: number;
    username: string;
  };
  text: string | null;
  image: string;
};

logOutButton.addEventListener("click", async () => {
  try {
    await fetch("/api/user/logout", {
      method: "POST",
    });

    window.location.href = "/";
    return;
  } catch {
    window.location.href = "/";
    return;
  }
});

async function loadPosts(): Promise<Post[]> {
  const res = await fetch("/api/post/timeline", {
    credentials: "include",
  });

  if (!res.ok) {
    throw new Error(`Could not load posts (${res.status})`); // I fucking hate these "`", fuck typescript for making me use them
  }

  return await res.json();
}

// build one post "card"
function makePost(post: Post) {
  const card = document.createElement("article");
  card.className = "post";
  const username = document.createElement("strong");
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

        await showPosts();
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

async function showPosts() {
  feed.setAttribute("aria-busy", "true");

  try {
    const posts = await loadPosts();

    feed.replaceChildren();

    if (posts.length === 0) {
      showMessage("No posts yet.");
      return;
    }

    for (const post of posts) {
      feed.append(makePost(post));
    }

    const endMessage = document.createElement("p");
    endMessage.className = "feed-empty";
    endMessage.textContent = "No more posts.";
    feed.append(endMessage);
  } catch (error) {
    console.error(error);
    showMessage("Could not load posts.");
  } finally {
    feed.removeAttribute("aria-busy");
  }
}

function showMessage(message: string) {
  const p = document.createElement("p");
  p.className = "feed-empty";
  p.textContent = message;
  feed.replaceChildren(p);
}

postCancel.addEventListener("click", () => {
  postForm.hidden = true;
  postForm.reset();
  postError.textContent = "";
});

postForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  postError.textContent = "";

  const image = postImage.files?.[0];

  if (!image) {
    postError.textContent = "Choose a picture first.";
    return;
  }

  if (new TextEncoder().encode(postText.value).length > 200) {
    postError.textContent = "Caption is too long.";
    return; // I know the server also checks this, I added this just so that we could get a better error message than "Could not publish your post"
  }

  const data = new FormData();
  data.append("image", image);

  if (postText.value) {
    data.append("text", postText.value);
  }

  const submit = postForm.querySelector<HTMLButtonElement>(
    'button[type="submit"]',
  )!;

  submit.disabled = true;

  try {
    const response = await fetch("/api/post/create", {
      method: "POST",
      credentials: "include",
      body: data,
    });

    if (!response.ok) {
      throw new Error(`Post failed (${response.status})`);
    }

    postForm.reset();
    postForm.hidden = true;
    await showPosts();
  } catch (error) {
    console.error(error);
    postError.textContent = "Could not publish your post. Please try again.";
  } finally {
    submit.disabled = false;
  }
});

async function init() {
  let loggedIn = false;

  try {
    const res = await fetch("/api/user/get", { credentials: "include" });
    if (res.ok) {
      const user = await res.json();
      currentUserId = user.id;
      loggedIn = true;
    }
  } catch {
    // Not logged in if you can't connect (401).
  }

  authAction.hidden = false;

  if (loggedIn) {
    authAction.textContent = "Post";
    logOutButton.hidden = false;

    authAction.addEventListener("click", () => {
      postForm.hidden = false;
      postImage.focus();
    });
  } else {
    authAction.textContent = "Log in";

    authAction.addEventListener("click", () => {
      window.location.href = "/login.html";
    });
  }

  await showPosts();
}
init();
