@0x924d11f44b8362c0;

##
## S->C is a Server to Client packet
## C->S is a Client to Server packet
## S?C  is a packet that can go both ways
## D    is a data structure for use in other packets
##

# D    | Data for a login attempt
struct Login @0xced7308647d1b425 {
    username  @0 :Text; # the user's entered username
    password  @1 :Text; # the user's entered password
    signup    @2 :Bool; # if the user is signing up or not (true if signup)
    clientVer @3 :Text; # the client version to verify if it can connect properly
}

# C->S | Checking server version or attempting a login
struct EntryPoint @0xa2c8f82e1b9de16e {
    union {
        version      @0 :Text;  # for checking compatability with server
        loginAttempt @1 :Login; # for trying to log in or sign up
    }
}

# S->C | Responding to the client's entry request
struct EntryResponse @0xef44d5bcb133a45f {
    union {
        motd    @0 :Text; # Sent when a login attempt is valid
        version @1 :Text; # Sent when the version is accepted
        error   @2 :Text; # When an error occurs - invalid login or invalid version
    }
}

# D    | For storing information about an item
struct Item @0x95863d8c2442143d {
    name   @0 :Text;     # the name of the item
    level  @1 :UInt32;   # the level of the item
    itype  @2 :UInt32;   # the item's type
    rarity @3 :UInt32;   # the rarity of the item
    # the item will either have a damage stat or a defense stat
    union {
        damage  @4 :UInt32; # how much damage the item does
        defense @5 :UInt32; # how much defense the item gives
    }
}

# D    | For sending display information about an enemy in an encounter
struct Enemy @0x9a5fa929bed5b944 {
    name   @0 :Text;   # the enemy's display name
    race   @1 :Text;   # the enemy's race
    level  @2 :UInt32; # the enemy's level
    health @3 :UInt32; # the enemy's health
}

# D    | For storing information about an encounter
struct Encounter @0xc3dc06bc33514e85 {
    enemy @0 :Enemy; # the enemy's data
    union {
        attk  @1 :UInt32; # if the enemy attacks
        flee  @2 :Bool;   # If the player was able to flee or not
        win   @3 :Loot;   # if the player won the encounter
        lost  @4 :Bool;   # if the player lost
    }
}

# D    | For when an enemy has been defeated and the player is receiving loot
struct Loot @0xd647d69f6ebd790e {
    items @0 :List(Item); # the items gained in the victory
    exp   @1 :UInt32;     # experience gained in victory
}

# D    | For if an error occurs
struct Error @0xb7c0dd88336ed014 {
    error      @0 :Text; # The error message
    disconnect @1 :Bool; # If the client should be disconnected because of this error
}

# D    | For sending information about the player
struct PlayerData @0x8a793e2e80578a33 {
    level  @0 :UInt32; # The player's level
    exp    @1 :UInt32; # The player's exp
    region @2 :Text;   # The region the player is in
    health @3 :UInt32; # The player's current health
    steps  @4 :UInt32; # The total amount of steps of the player
}

# S->C | For an event from the server to the client
# Usually run after a step
struct SEvent @0xa3a26618dd4da69f {
    union {
        disconnect @0 :Bool; # if the server is telling the client to disconnect
        keepalive  @1 :UInt64;     # for handling the keepalive system
        event      @2 :Text;       # info for the client to print out
        gainExp    @3 :UInt32;     # player gains experience
        findItem   @4 :Item;       # player finds an item
        encounter  @5 :Encounter;  # player encounters an enemy
        inventory  @6 :List(Item); # player requested to view inventory
        itemView   @7 :Item;       # player views an item in the inventory
        update     @8 :PlayerData; # Information about the player
        error      @9 :Error;      # an error if one occurred
    }
}

# C->S | For an event from the client to the server
# For telling the server of inputs, like taking a step
struct CEvent @0xd96b16669441a8da {
    union {
        disconnect @0 :Bool;   # if the client needs to disconnect or not
        keepalive  @1 :UInt64; # for handling the keepalive system
        step       @2 :Bool;   # player takes a step
        rqstUpdate @3 :Void;   # request update
        openInv    @4 :Bool;   # player opens inventory
        dropItm    @5 :Text;   # name of an item to drop in the inventory
        inspectItm @6 :Text;   # name of an item to inspect in inventory
        attack     @7 :Bool;   # player tries to attack
        tryFlee    @8 :Bool;   # player tries to flee
        error      @9 :Error;  # if an error has occurred
    }
}