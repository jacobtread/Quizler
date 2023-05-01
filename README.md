<img src="assets/logo.svg" width="100%" height="80px">

# Quizler


> **Note**
> This project is still in its very early stages of development so its not yet released or functional

This is the improved version of my [Quizler](https://github.com/jacobtread/Quizler) app which is intended to be a real time quiz application similar to [Kahoot](https://kahoot.com/) but *OpenSource* and able to be played offline over LAN.

This version aims to improve upon the downfalls of the previous app (Written in Go) along with providing new features. I aim to make this version more stable and performant using my better understanding of the **Rust** language.

This new version makes use of **Rust** for the backend and **Svelte** for the frontend. The previous version used **Go** for the backend and **VueJS** for the frontend.


## Planned New Features

- Multiple question types, this new version aims to add multiple new question types to choose from:
    - Question with multiple choice but only 1 is right
    - Multiple choice question with multiple right answers
    - Image question where you select a region on the image
- Customizable timings, you will be able to customize the time given for each question, the time between questions, the time available for bonus scoring.
- Customizible scoring, you will be able to decide the minimum and maximum score that can be earned for a question based on the time it took to answer along with customizing the amount of score gained if answered within the bonus period.
