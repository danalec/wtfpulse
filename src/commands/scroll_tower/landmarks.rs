// src/commands/scroll_tower/landmarks.rs

/// Represents the type/genre of the landmark for filtering or icon display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Bio,
    Structure,
    Fiction,
    Space,
    Tech, // New category for IT heritage
    Meme,
}

/// Represents a physical landmark or object to climb via scrolling.
#[derive(Debug, Clone)]
pub struct Landmark {
    /// Display name of the landmark
    pub name: &'static str,
    /// Physical height in meters (threshold to unlock)
    pub height_meters: f64,
    /// The category for UI coloring
    #[allow(dead_code)]
    pub category: Category,
    /// A fun fact or description to show upon conquering
    pub description: &'static str,
    /// ASCII Art representation (raw string literal)
    pub ascii_art: &'static str,
}

pub const LANDMARKS: &[Landmark] = &[
    Landmark {
        name: "Rubber Duck 🦆",
        height_meters: 0.1,
        category: Category::Tech,
        description: "The best listener you know. Has heard all your bad code logic.",
        ascii_art: r#"
          _
         (.)>
        __|__
       /     \
      |_______|
    "#,
    },
    Landmark {
        name: "Ferris the Crab 🦀",
        height_meters: 0.3,
        category: Category::Tech,
        description: "The Rust mascot. He promises memory safety if you keep scrolling.",
        ascii_art: r#"
         _~^~^~_
     \) /  o o  \ (/
       '_   -   _'
       / '-----' \
    "#,
    },
    Landmark {
        name: "Tux the Penguin 🐧",
        height_meters: 0.8,
        category: Category::Tech,
        description: "The kernel guardian. He grants you sudo access to climb higher.",
        ascii_art: r#"
           .--.
          |o_o |
          |:_/ |
         //   \ \
        (|     |)
       /'\_   _/`\
       \___)=(___/
    "#,
    },
    Landmark {
        name: "The Unix Cow (Cowsay) 🐮",
        height_meters: 1.5,
        category: Category::Tech,
        description: "Moo. I have no idea what I'm doing, but I can pipe output.",
        ascii_art: r#"
     ^__^
     (oo)\_______
     (__)\       )\/\
         ||----w |
         ||     ||
    "#,
    },
    Landmark {
        name: "42U Server Rack 🖥️",
        height_meters: 2.0,
        category: Category::Tech,
        description: "A standard 19-inch rack. Contains 42 units of pure spaghetti cabling.",
        ascii_art: r#"
       +------------------+
       |[  [SWITCH-01]   ]|
       |[  [ROUTER-01]   ]|
       |[  [PATCH-PNL]   ]|
       |[  [SRV-APP01]   ]|
       |[  [SRV-APP02]   ]|
       |[  [SRV-DB-01]   ]|
       |[  [SRV-DB-02]   ]|
       |[  [STORAGE-1]   ]|
       |[  [STORAGE-2]   ]|
       |[  [UPS-MAIN ]   ]|
       |[________________]|
       +------------------+
    "#,
    },
    Landmark {
        name: "Giraffe (Adult Male)",
        height_meters: 5.5,
        category: Category::Bio,
        description: "The tallest living terrestrial animal. Eats acacias and judges your code.",
        ascii_art: r#"
             oo
            / _
           / /
          / /
         / /
        / /
       / /
      / /
     / /
    / /
   / /
  / /   _
 / /  _/ \
 | | /  _ \
 | |/  / \ \
 |    /   \ \
 |___/     \_\
  | |       | |
  | |       | |
    "#,
    },
    Landmark {
        name: "Brontosaurus",
        height_meters: 6.0,
        category: Category::Bio,
        description: "The 'Thunder Lizard'. Proved you can be huge and still have a small header file.",
        ascii_art: r#"
                __
               / _)
      _.----._/ /
     /         /
  __/ (  | (  |
 /__.-'|_|--|_|
"#,
    },
    Landmark {
        name: "Great Wall of China 🇨🇳",
        height_meters: 8.8,
        category: Category::Structure,
        description: "Not visible from space, but definitely visible in your terminal. (Avg Height)",
        ascii_art: r#"
        _   _   _   _   _
       | |_| |_| |_| |_| |
      _|                 |_
     |                     |
    _|                     |_
   |_________________________|
   |  |  |  |  |  |  |  |  |
   |__|__|__|__|__|__|__|__|
"#,
    },
    Landmark {
        name: "Sauroposeidon",
        height_meters: 18.0,
        category: Category::Bio,
        description: "The 'Earthquake God Lizard'. Neck longer than your git history.",
        ascii_art: r#"
                  __
                 (  )
                  )(
                 (  )
                  )(
                 (  )
                  )(
              ___(  )
             /     /
            /     /
           /  |  |
          /   |  |
"#,
    },
    Landmark {
        name: "RX-0 Unicorn Gundam 🇯🇵",
        height_meters: 19.7,
        category: Category::Fiction,
        description: "Destroy Mode activated. Hope your scroll wheel is Newtype-compatible.",
        ascii_art: r#"
             \ /
           --(_)--
             / \
            /| |\
           /_|_|_\
          |  |  |
          |  |  |
         /|  |  |\
        / |  |  | \
       /  |  |  |  \
      /   |__|__|   \
     /___/       \___\
"#,
    },
    Landmark {
        name: "Blue Whale (Vertical)",
        height_meters: 30.0,
        category: Category::Bio,
        description: "If you balanced a Blue Whale on its tail. Don't try this at home.",
        ascii_art: r#"
              .
             :
            : :
           :   :
          :     :
         :       :
        :_________:
        |         |
        |         |
        |         |
        |    O    |
        |         |
        \       /
         \     /
          \   /
           \ /
            V
"#,
    },
    Landmark {
        name: "Stack of Unfixed Bugs",
        height_meters: 64.0,
        category: Category::Meme,
        description: "Based on the average thickness of a printed Jira ticket (0.1mm) * 640k items.",
        ascii_art: r#"
      [TODO]
     [FIXME]
    [DEPRECATED]
   [SEGFAULT]
  [WRAP_ERR]
 [UNWRAP()]
[CLONE().CLONE()]
"#,
    },
    Landmark {
        name: "Statue of Liberty 🇺🇸",
        height_meters: 93.0,
        category: Category::Structure,
        description: "Give me your tired, your poor, your huddled masses yearning to scroll free.",
        ascii_art: r#"
             |
             |
           _/^\_
          <     >
           /.-.\
          `/   \`
          /_____\
         |       |
         |       |
         |_______|
"#,
    },
    Landmark {
        name: "Godzilla (Monsterverse) 🦖",
        height_meters: 120.0,
        category: Category::Fiction,
        description: "King of the Monsters. Powered by radiation, just like your eyes.",
        ascii_art: r#"
             ,
            /|   _
          _/_|__/_\_
         /          \
        |  (o)  (o)  |
        C            |
         |   ____   |
        /|  /    \  |\
       / | |      | | \
      /  | |      | |  \
     /   |_|      |_|   \
    /___/            \___\
"#,
    },
    Landmark {
        name: "Great Pyramid of Giza 🇪🇬",
        height_meters: 138.5,
        category: Category::Structure,
        description: "Aliens? Slaves? No, just a lot of `cargo build --release`.",
        ascii_art: r#"
             /\
            /  \
           /    \
          /      \
         /________\
        /__________\
       /____________\
"#,
    },
    Landmark {
        name: "Eiffel Tower 🇫🇷",
        height_meters: 330.0,
        category: Category::Structure,
        description: "A permanent monument to the temporary scaffolding used to build it.",
        ascii_art: r#"
             /\
            /**\
           /****\
          /      \
         /  ____  \
        /  /    \  \
       /__/______\__\
          /|    |\
         / |    | \
        /__|____|__\
"#,
    },
    Landmark {
        name: "Tokyo Tower 🇯🇵",
        height_meters: 333.0,
        category: Category::Structure,
        description: "Higher than Eiffel and painted orange for aviation safety. Kaiju magnet.",
        ascii_art: r#"
             |
            _A_
           /   \
          /_____\
         /       \
        /_________\
       /           \
      /_____________\
     /               \
    /_________________\
"#,
    },
    Landmark {
        name: "Empire State Building 🇺🇸",
        height_meters: 443.0,
        category: Category::Structure,
        description: "The spot where King Kong fell for a blonde.",
        ascii_art: r#"
             |
            _|_
           |   |
           |   |
           |___|
          /     \
         |       |
         |       |
         |_______|
        /         \
       |           |
       |___________|
      /             \
     |_______________|
"#,
    },
    Landmark {
        name: "Tokyo Skytree 🇯🇵",
        height_meters: 634.0,
        category: Category::Structure,
        description: "The world's tallest tower. Musashi (634) stands watch over Sumida.",
        ascii_art: r#"
             |
             |
            |||
            |||
           /|||\
          / ||| \
         /__|||__\
"#,
    },
    Landmark {
        name: "Burj Khalifa 🇦🇪",
        height_meters: 828.0,
        category: Category::Structure,
        description: "The world's tallest building. A testament to human engineering and excess.",
        ascii_art: r#"
             |
             |
             |
             |
             |
            / \
            | |
           /| |\
          / | | \
         /  | |  \
        /__ |_| __\
"#,
    },
    Landmark {
        name: "Barad-dûr (Sauron's Tower) 👁️",
        height_meters: 1_500.0,
        category: Category::Fiction,
        description: "A great eye, lidless, wreathed in flame. It sees your browser history.",
        ascii_art: r#"
            ( )
           (   )
          (  O  )
           (   )
            | |
           /   \
          /     \
         |       |
         |   _   |
        /|  | |  |\
       / |  | |  | \
      /  |__| |__|  \
     /_______________\
    /_________________\
"#,
    },
    Landmark {
        name: "Cumulonimbus Cloud ☁️",
        height_meters: 2_000.0,
        category: Category::Bio, // Nature?
        description: "A massive storm cloud. Don't scroll too fast or you'll get struck by lightning.",
        ascii_art: r#"
             .--.
          .-(    ).
         (  .  )   )
        (        )  )
       (_____________)
          /  /  /
           /  /
"#,
    },
    Landmark {
        name: "Mount Fuji 🇯🇵",
        height_meters: 3_776.0,
        category: Category::Bio, // Or Structure? It's a mountain (Bio/Nature). Category::Bio seems best fit among existing (Bio, Structure, Fiction, Space, Tech, Meme). Or maybe add 'Nature'? Bio is close enough for now (Giraffe, Whale).
        description: "The sacred mountain. Ideally viewed from a Shinkansen window.",
        ascii_art: r#"
             /\
            /~~\
           /    \
          /      \
         /________\
"#,
    },
    Landmark {
        name: "HAL 9000 Logic Center 🔴",
        height_meters: 4_000.0, // Arbitrary "Depth" of the ship, or height of the Monolith
        category: Category::Tech,
        description: "I'm afraid I can't let you scroll that, Dave.",
        ascii_art: r#"
         .[___________].
         | [ ] [ ] [ ] |
         | [ ] [O] [ ] |
         | [ ] [ ] [ ] |
         |_____________|
    "#,
    },
    Landmark {
        name: "Everest Base Camp ⛺",
        height_meters: 5_364.0,
        category: Category::Structure,
        description: "The starting point for glory. Or for turning back to get coffee.",
        ascii_art: r#"
            _
           / \
          /   \
         /  _  \
        /__/ \__\  ( )
           | |    / | \
    "#,
    },
    Landmark {
        name: "Mt. Everest 🇳🇵",
        height_meters: 8_849.0,
        category: Category::Structure,
        description: "The death zone. Oxygen is low, much like your motivation on Monday morning.",
        ascii_art: r#"
              /\
             /  \
            /    \   /\
           /      \ /  \
          /        /    \
         /________/______\
"#,
    },
    Landmark {
        name: "Commercial Airliner ✈️",
        height_meters: 11_000.0,
        category: Category::Tech,
        description: "Cruising altitude. Please fasten your seatbelt and return your tray table to the upright position.",
        ascii_art: r#"
           _
         -=\`\
     |\ ____\_\__
    -=\c`""""""" "`)
       `~~~~~/ /~~`
        -==/ /
          '-'
"#,
    },
    Landmark {
        name: "Olympus Mons (Mars) 🪐",
        height_meters: 21_900.0,
        category: Category::Space,
        description: "The tallest mountain in the solar system. Makes Everest look like a speed bump.",
        ascii_art: r#"
             /^\
           /     \
          /       \
      ___/         \___
     /                 \
    |___________________|
"#,
    },
    Landmark {
        name: "Stratosphere Jump 🪂",
        height_meters: 39_045.0,
        category: Category::Tech,
        description: "Felix Baumgartner's record jump. The ultimate drop test.",
        ascii_art: r#"
             O
            /|\
           / | \
             |
            / \
           /   \
"#,
    },
    Landmark {
        name: "High-Altitude Balloon 🎈",
        height_meters: 53_000.0,
        category: Category::Tech,
        description: "The BU60-1 balloon record. Thin air, great view, zero pressure.",
        ascii_art: r#"
          .---.
        .'     '.
       /         \
      :           :
       \         /
        '.     .'
          `._.`
           | |
          [___]
"#,
    },
    Landmark {
        name: "Shooting Star 🌠",
        height_meters: 70_000.0,
        category: Category::Space,
        description: "A meteor burning up in the mesosphere. Make a wish!",
        ascii_art: r#"
                                  *
                                --+--
                                  *
                                . '
                              . '
                            . '
                          . '
                        . '
                      . '
                    . '
                  . '
                . '
              . '
            . '
          . '
        . '
      . '
    . '
   '
"#,
    },
    Landmark {
        name: "Aurora Borealis 🌌",
        height_meters: 85_000.0,
        category: Category::Space,
        description: "Solar wind hitting the atmosphere. Nature's RGB lighting.",
        ascii_art: r#"
           .   *   .   *   .   *   .   *   .
          *   .   *   .   *   .   *   .   *
       .     .     .     .     .     .     .
      (   *   (   *   (   *   (   *   (   *
       )     . )     . )     . )     . )
      (   .   (   .   (   .   (   .   (
       )     * )     * )     * )     * )
      (   *   (   *   (   *   (   *   (
       )     . )     . )     . )     . )
       |   .   |   .   |   .   |   .   |
       |       |       |       |       |
       |   *   |   *   |   *   |   *   |
       |       |       |       |       |
"#,
    },
    Landmark {
        name: "Kármán Line (Space) 🚀",
        height_meters: 100_000.0,
        category: Category::Space,
        description: "100km vertical. You have officially left Earth.",
        ascii_art: r#"
             ^
            / \
           | W |
           | T |
           | F |
          /| | |\
         /_|_|_|_\
           / \
          /   \
         (_____)
"#,
    },
    Landmark {
        name: "Death Star (Diameter) 🌑",
        height_meters: 120_000.0,
        category: Category::Fiction,
        description: "That's no moon... it's a very expensive scrolling target.",
        ascii_art: r#"
          .          .
      .-""""""""""""""-.
    .'       ___        `.
   /       .'   `.        \
  ;        :     :         ;
  |        `.__.'          |
  |        ___________     |
  ;       |___________|    ;
   \                      /
    `.                  .'
      `._            _.'
         `""""""""""`
"#,
    },
    Landmark {
        name: "International Space Station 🛰️",
        height_meters: 400_000.0,
        category: Category::Space,
        description: "Orbiting at 27,600 km/h. Try not to get scroll-sick.",
        ascii_art: r#"
       |[]|       |[]|
       |[]|-------|[]|
       |[]|   |   |[]|
       |[]|---|---|[]|
              |
             [O]
"#,
    },
    Landmark {
        name: "James Webb Space Telescope 🔭",
        height_meters: 1_500_000.0,
        category: Category::Space,
        description: "Looking at the dawn of the universe. Or just your commit history.",
        ascii_art: r#"
          \ | /
         __\|/__
        /  / \  \
       |  | O |  |
        \  \ /  /
         -------
         /_____\
          |   |
          |___|
"#,
    },
];
