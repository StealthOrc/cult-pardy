<script>
    // Sample data for players including icons
    let players = [
        { id: 1, name: 'Player 1', icon: '👤' },
        { id: 2, name: 'Player 2', icon: '👥' },
        { id: 3, name: 'Player 3', icon: '👑' },
    ];
</script>

<div class="player-container">
    {#each players as player}
        <div class="player-card">
            <div class="player-icon">{player.icon}</div>
            <h3>{player.name}</h3>
        </div>
    {/each}
</div>

<style>
    .player-container {
        display: flex;
        justify-content: center;
        align-items: flex-end;
        position: fixed;
        bottom: 20px;
        width: 100%;
        z-index: 1;
    }

    .player-card {
        background-color: #f1f1f1;
        border: 1px solid #ddd;
        border-radius: 5px;
        padding: 10px;
        margin: 0 10px;
        box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
        display: flex;
        align-items: center;
    }

    .player-icon {
        font-size: 24px;
        margin-right: 10px;
    }

    .player-card h3 {
        margin: 0;
        font-size: 18px;
    }
</style>
