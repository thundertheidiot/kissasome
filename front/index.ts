import "./styles/index.css";
import type { Post } from "./post.ts";
import { makePostShared } from "./post.ts";

const authAction = document.querySelector<HTMLButtonElement>("#auth-action")!;
const logOutButton = document.querySelector<HTMLButtonElement>("#log-out")!;
const feed = document.querySelector<HTMLElement>("#feed")!;
const postForm = document.querySelector<HTMLFormElement>("#post-form")!;
const postImage = document.querySelector<HTMLInputElement>("#post-image")!;
const postText = document.querySelector<HTMLTextAreaElement>("#post-text")!;
const postError = document.querySelector<HTMLParagraphElement>("#post-error")!;
const postCancel = document.querySelector<HTMLButtonElement>("#post-cancel")!;
const status = document.querySelector<HTMLParagraphElement>("#status")!;

let currentUserId: number | null = null;

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

const makePost = (post: Post) =>
  makePostShared(post, currentUserId, postDeleted);

async function postDeleted() {
  await showPosts();
  feed.focus();
  status.textContent = "Post deleted.";
  // this status is shown to screen reader users
}

async function loadPosts(): Promise<Post[]> {
  const res = await fetch("/api/post/timeline", {
    credentials: "include",
  });

  if (!res.ok) {
    throw new Error(`Could not load posts (${res.status})`); // I fucking hate these "`", fuck typescript for making me use them
  }

  return await res.json();
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

function showPostError(
  message: string,
  field: HTMLInputElement | HTMLTextAreaElement,
) {
  postError.textContent = message;
  field.setAttribute("aria-invalid", "true");
  field.focus();
}

function clearPostError() {
  postError.textContent = "";
  for (const field of [postImage, postText]) {
    field.removeAttribute("aria-invalid");
  }
}

postCancel.addEventListener("click", () => {
  postForm.hidden = true;
  postForm.reset();
  clearPostError();
  authAction.focus();
});

postForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  clearPostError();

  const image = postImage.files?.[0];

  if (!image) {
    showPostError("Choose a picture first.", postImage);
    return;
  }

  const textLength = new TextEncoder().encode(postText.value).length;

  if (textLength > 200) {
    showPostError(
      `Caption is too long (${textLength}/200 characters).`,
      postText,
    );
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
    feed.focus();
    status.textContent = "Post published.";
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
