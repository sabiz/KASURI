<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  const INVOKE_SEARCH_APPLICATION = "search_application";
  const INVOKE_CHANGED_CONTENT_SIZE = "changed_content_size";

  interface Application {
    name: string;
    app_id: string;
  }

  let searchQuery = $state("");
  let suggestions = $state<Application[]>([]);
  let showSuggestions = $state(true);
  let selectedSuggestionIndex = $state(-1);
  let suggestionListElement = $state<HTMLElement | null>(null);
  let searchFormElement: HTMLFormElement | null = null;
  let mainElement: HTMLElement | null = null;
  let searchInputClass = $state("");
  let lastSendContentSize = 0;

  $effect(() => {
    searchInputClass =
      showSuggestions && suggestions.length > 0
        ? "border-x-1 border-t-1 border-b-0 rounded-t-lg"
        : "border-1 rounded-lg";
    send_content_size();
  });

  /*
   * handle the input event of the search input field.
   */
  async function handleSearchInput() {
    if (searchQuery.trim() === "") {
      suggestions = [];
      showSuggestions = false;
      selectedSuggestionIndex = -1;
      return;
    }

    let result = await invoke(INVOKE_SEARCH_APPLICATION, {
      query: searchQuery,
    });
    suggestions = result as Application[];
    showSuggestions = true;
    selectedSuggestionIndex = 0;
    await send_content_size();
  }

  function handleKeyDown(event: KeyboardEvent) {
    console.log("Key pressed:", event.key);

    if (event.key === "Escape") {
      event.preventDefault();
      // TODO Close window
    }

    switch (event.key) {
      case "ArrowDown":
        moveSuggestion(1);
        break;
      case "ArrowUp":
        moveSuggestion(-1);
        break;
    }
  }
  function moveSuggestion(direction: number) {
    if (suggestions.length === 0 || !showSuggestions) return;
    selectedSuggestionIndex =
      (selectedSuggestionIndex + direction + suggestions.length) %
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
   * This function handles the selection of a suggestion from the list.
   */
  function selectSuggestion(suggestion: string) {
    searchQuery = suggestion;
    showSuggestions = false;
    if (searchFormElement) {
      searchFormElement.dispatchEvent(
        new Event("submit", { cancelable: true }),
      );
    }
  }

  // Handle search form submission
  function handleSearch(event: Event) {
    event.preventDefault();
    console.log("Searching for:", searchQuery);
    // Here you would typically process the search
    showSuggestions = false;
  }

  // send the content size to the backend
  async function send_content_size() {
    if (lastSendContentSize === mainElement?.offsetHeight) {
      return;
    }
    await invoke(INVOKE_CHANGED_CONTENT_SIZE, {
      contentHeight: mainElement?.offsetHeight,
    });
    lastSendContentSize = mainElement?.offsetHeight || 0;
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
      onsubmit={handleSearch}
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
          searchInputClass,
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
          oninput={handleSearchInput}
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
      {#if showSuggestions && suggestions.length > 0}
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
              onmousedown={() => selectSuggestion(suggestion.app_id)}
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
