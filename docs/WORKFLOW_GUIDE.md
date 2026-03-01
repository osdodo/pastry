# Workflow Guide

This document explains how to build and run workflows in Pastry.

## What Is a Workflow

A workflow is a node graph that processes data automatically after a trigger.

## Node Types

- **Hotkey**: starts the workflow from a global shortcut.
- **Script**: runs JavaScript to transform data.
- **Clipboard**: reads from or writes to system clipboard.
- **File Write**: writes output content to a local file.

## Basic Setup

1. Open the Workflow page.
2. Create a new workflow.
3. Add nodes and connect them.
4. Configure each node input/output.
5. Enable the workflow.

## Trigger and Run

- If a `Hotkey` node exists, press the assigned shortcut.
- You can also run from the workflow list/editor if supported.

## Practical Example

`Clipboard Read -> Script -> Clipboard Write`

- Read selected text from clipboard.
- Convert content in a script (e.g. Base64, hash, format).
- Write result back to clipboard.
