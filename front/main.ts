import { setupButton } from "./counter";

// ! is typescript null assert operator
// it only makes the compiler not yell at you
setupButton(document.querySelector<HTMLButtonElement>('#counter')!);

document.querySelector<HTMLButtonElement>('#fetch')?.addEventListener('click', async () => {
  const dataElement = document.querySelector<HTMLParagraphElement>('#data')!;

  const url = "/api/hello";

  try {
	const resp = await fetch(url);
	if (!resp.ok) {
	  throw new Error(`Failed, status: ${resp.status}`);
	}

	const result = await resp.json();
	dataElement.innerHTML = JSON.stringify(result);
  } catch (error) {
	console.error(error);
  }
});
