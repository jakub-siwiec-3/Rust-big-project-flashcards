# Flashcards App

A spaced repetition learning application built with Rust and egui, implementing the SuperMemo SM-2 algorithm for efficient memorization.
Note: the time simulation is simplified and based only on the "Next day" button usage. Use this button to test how the spaced repetition algorithm affects the flashcards' repetition.

## Overview

This application helps users learn vocabulary and concepts through flashcards with scientifically-proven spaced repetition. Cards are scheduled for review based on user performance, optimizing long-term retention.

## Features

- **Deck Management**: Create multiple flashcard decks, add/edit cards
- **Spaced Repetition**: SM-2 algorithm calculates optimal review intervals based on performance (quality ratings 0-5)
- **Learning Sessions**: Interactive study mode with definition reveal and self-assessment
- **Multi-Round Review**: Cards rated below 3 automatically cycle until mastered
- **Persistent Storage**: SQLite database stores decks, cards, and review statistics
- **Import/Export**: JSON format for sharing
- **Time Simulation**: "Next Day" feature for testing scheduling algorithm

## Implementation

### Database Schema

- **decks**: Deck metadata
- **flashcards**: Terms and definitions with deck association
- **review_data**: SM-2 parameters (E-Factor, interval, repetitions, next review date)
- **app_state**: Current simulated date

## Summary
Successfully implemented the main screen and learning session screen in the UI. Models for flashcards, decks etc. with a local database, import and export of decks, learning session flow and spaced repetition algorithm synchronized with the learning session. Rust features: closures, traits, generics, Result/Option types, iterators, ownership system, Arc/Mutex concurrency, derive macros, error handling, modules.
There is no async and the feature of images in the flashcards was not implemented. 