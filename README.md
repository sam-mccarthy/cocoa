# Cocoa

A fun, general-purpose Discord bot written in Rust.

## How do I run it?

There's a template `docker-compose.yaml` in the files - you can use that to set up Mongo, and then set up the bot.

## Why?

Many years ago, when I was but a child, one of my first major development projects was a general-purpose Discord bot -
it was written in C# using Discord.NET, and was a fantastic learning experience.

Recently, I've gained a renewed fondness for the idea, and decided to take another crack at it.

## Development

The bot is currently still very early in development, and as such, there are many shortcuts that I've made for the sake
of creating a proof-of-concept. The main issues right now are overall command / UX polish, error handling, logging, and
command implementation.

Polish, logging, and error handling will all be fixed in an upcoming rewrite of the LastFM flavor, which holds most of
the commands - the LastFM section is just really messy.

Command implementation will come with time - I'm still not 100% sure on what I want to implement in the bot, and I'm not
sure how I want to implement it either - I'd like some sort of economy, but it'll take some time to figure out the
balance of it.

I also think that some things could be streamlined / cleaned up a bit, and the code could do with some commenting.

## Will this bot become available publicly?

Not yet - I'd like to finish ironing out the details and making sure there aren't any major issues.