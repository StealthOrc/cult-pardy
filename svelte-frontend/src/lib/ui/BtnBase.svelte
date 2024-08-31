<script lang="ts">
	import type { BaseButtonProps } from '$lib/types';
    
    let { onclick, text, Icon  }: BaseButtonProps = $props();	

</script>

<button {onclick} class="cult-btn-menu fixed top-2 left-2 m-2">
    <div class="flex flex-row">{@render Icon()}{text}</div>
</button>