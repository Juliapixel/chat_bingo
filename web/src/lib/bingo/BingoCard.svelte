<script lang="ts">
    import type { BingoCardData, BingoSquare } from "./bingo_data.ts";

    export let data: BingoCardData;

    function handleClick(localData: BingoSquare) {
        return () => {
            if (localData.selected) {
                localData.selected = false
            } else {
                localData.selected = true
            }
            // do this so that svelte rerenders it
            data = data
            console.log(localData)
        }
    }
</script>

<table class="border-2">
    <caption class="border">
        BINGO
    </caption>
    {#each {length: data.size} as _, i}
        <tr>
            {#each {length: data.size} as _, j}
                <td class="border">
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div class:bg-red-500={data.getData(j, i)?.selected} class="flex items-center justify-center text-center w-24 h-24" on:click={handleClick(data.getData(j, i))}>
                        <p class="max-w-full">
                            {data.getData(j, i)?.text || ""}
                        </p>
                    </div>
                </td>
            {/each}
        </tr>
    {/each}
</table>
