<script lang="ts">
  import type { Application } from "../core/backend";
  import { Backend } from "../core/backend";

  let mainElement: HTMLElement | null = null;
  let searchQuery = $state("");
  let suggestions = $state<Application[]>([]);
  let selectedSuggestionIndex = $state(-1);
  let suggestionListElement = $state<HTMLElement | null>(null);
  let searchFormElement: HTMLFormElement | null = null;
  let queryInputClass = $state("");

  let backend = new Backend();

  $effect.pre(() => {
    // Set the initial state of the search query
    updateSelectedSuggestionIndex(true);
  });

  $effect(() => {
    updateQueryInputClass();
    backend.sendContentSize(mainElement?.clientHeight || 0);
  });

  /*
   * updates the class of the search input field based on the
   * state of the suggestions.
   */
  function updateQueryInputClass() {
    if (suggestions.length > 0) {
      queryInputClass = "border-x-1 border-t-1 border-b-0 rounded-t-lg";
    } else {
      queryInputClass = "border-1 rounded-lg";
    }
  }

  /*
   * updates the selected suggestion index based on the
   * current state of the suggestions.
   */
  function updateSelectedSuggestionIndex(
    byEffect: boolean = false,
    moveDirection: number = 0,
  ) {
    if (suggestions.length === 0) {
      selectedSuggestionIndex = -1;
      return;
    }
    if (byEffect) {
      selectedSuggestionIndex = 0;
      return;
    }
    selectedSuggestionIndex =
      (selectedSuggestionIndex + moveDirection + suggestions.length) %
      suggestions.length;

    if (!suggestionListElement) return;
    const selectedElement = suggestionListElement.children[
      selectedSuggestionIndex
    ] as HTMLElement;
    if (!selectedElement) return;
    selectedElement.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
    });
  }

  /*
   * closes the search window.
   */
  function closeMe() {
    backend.close();
    searchQuery = "";
    handleQueryInput();
  }

  /*
   * handle the input event of the query input field.
   */
  async function handleQueryInput() {
    if (searchQuery.trim() === "") {
      suggestions = [];
      return;
    }
    let result = await backend.searchApplication(searchQuery);
    suggestions = result as Application[];
  }

  /*
   * handle the keydown event of the window.
   */
  function handleKeyDown(event: KeyboardEvent) {
    // console.log("Key pressed:", event.key);

    switch (event.key) {
      case "Escape":
        closeMe();
        break;
      case "ArrowDown":
        updateSelectedSuggestionIndex(false, 1);
        break;
      case "ArrowUp":
        updateSelectedSuggestionIndex(false, -1);
        break;
    }
  }

  // Handle search form submit
  function handleSubmit() {
    if (
      suggestions.length === 0 ||
      selectedSuggestionIndex > suggestions.length
    )
      return;
    const selectedSuggestion = suggestions[selectedSuggestionIndex];
    if (!selectedSuggestion) return;
    console.log("Selected suggestion:", selectedSuggestion);
    backend.launch(selectedSuggestion);
    closeMe();
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<main
  class={["w-full", "bg-transparent", "overflow-hidden"]}
  bind:this={mainElement}
>
  <div class={["w-full"]}>
    <form
      class={["w-full", "relative"]}
      onsubmit={handleSubmit}
      bind:this={searchFormElement}
    >
      <div
        data-tauri-drag-region
        class={[
          "w-full",
          "flex",
          "relative",
          "bg-(--color-bg)",
          "border-solid",
          "border-(--color-line)",
          "pr-3",
          queryInputClass,
        ]}
      >
        <input
          type="text"
          class={[
            "w-[calc(100%-10px)]",
            "pl-3",
            "pr-3",
            "m-3",
            "text-2xl",
            "ease-linear",
            "bg-(--color-bg)",
            "placeholder:text-(--color-line)",
            "text-(--color-text)",
            "shadow-(--shadow-base)",
            "outline-none",
          ]}
          placeholder="Application name..."
          bind:value={searchQuery}
          oninput={handleQueryInput}
        />
        <button
          type="submit"
          class={[
            "absolute",
            "right-[15px]",
            "top-[50%]",
            "transform-[translateY(-50%)]",
            "background-none",
            "border-none",
            "padding-[8px]",
            "cursor-pointer",
            "display-flex",
            "align-items-center",
            "justify-content-center",
          ]}
          aria-label="Search"
        >
          <svg
            viewBox="0 0 24 24"
            class={["w-[52px]", "h-[52px]", "text-(--color-text)"]}
          >
            <path
              fill="currentColor"
              d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"
            />
          </svg>
        </button>
      </div>
      {#if suggestions.length > 0}
        <ul
          class={[
            "w-full",
            "z-10",
            "px-[1px]",
            "border-x-1",
            "border-b-1",
            "border-solid",
            "border-(--color-line)",
            "rounded-b-lg",
            "overflow-auto",
            "max-h-[15em]",
          ]}
          bind:this={suggestionListElement}
        >
          {#each suggestions as suggestion, index}
            <button
              type="button"
              class={[
                "w-full",
                "text-2xl",
                "text-left",
                "pl-6",
                "py-2",
                index === selectedSuggestionIndex
                  ? "bg-(--color-bg-light)"
                  : "bg-(--color-bg-lightx2)",
                "text-(--color-text)",
                "last:rounded-b-lg",
              ]}
              onmousedown={handleSubmit}
              onfocus={() => {
                selectedSuggestionIndex = index;
              }}
              onmouseover={() => {
                selectedSuggestionIndex = index;
              }}
            >
              {suggestion.name}
            </button>
          {/each}
        </ul>
      {/if}
    </form>
  </div>
</main>

<style lang="postcss">
  @reference "tailwindcss";
</style>
