def enter(proxy) -> bool:
    if proxy.has_level_30_character():
        proxy.open_npc(2007)

    proxy.block_portal()
    return True