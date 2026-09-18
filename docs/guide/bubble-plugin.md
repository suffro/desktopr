---
title: Bubble.io Plugin
description: Use the Desktopr Bubble.io plugin to access native desktop features in Bubble apps.
---

# Desktopr Bubble.io Plugin (BETA)

The **Desktopr Bubble Plugin** lets you use Desktopr native desktop features directly inside a [Bubble](https://bubble.io) application.

Bubble.io is a visual web app builder that allows you to create applications and workflows without writing most of the frontend code manually.  
With the Desktopr plugin, you can connect your Bubble app to the Desktopr bridge and use native desktop capabilities through Bubble elements, states, actions, and events.

The plugin is designed for Bubble developers who want to add Desktopr capabilities to their web app.

## How It Works

The plugin exposes Bubble elements that map to Desktopr bridge modules.

This means you can:

- place an element in your Bubble page
- call Desktopr methods with the element actions from Bubble workflows
- react to the element events
- read exposed states from the element

For most use cases, the plugin provides structured elements that are easier to use in Bubble.

For advanced use cases, it also includes a **raw advanced caller** that lets you manually invoke bridge methods with typed arguments.

## Who It Is For

This plugin is useful if you:

- build apps with Bubble
- want native desktop functionality
- want a simple Bubble actions for calling the Desktopr bridge
- want to prototype and ship desktop behavior faster

## Notes

:::tip
Use the dedicated plugin elements for the best Bubble experience. They expose typed actions, states, and events that are easier to use in workflows.
:::

:::info
An advanced raw caller is also available for manual bridge calls when you need more flexibility.
:::

:::warning
This Bubble.io Plugin is currently in **beta**.
:::

:::warning
Some native features may behave differently depending on the operating system, so always test your workflows on the target platform.
:::