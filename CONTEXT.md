# Sudoku

Offline desktop Sudoku game managing puzzle generation, interactive solving, scoring, and local play history.

## Language

### Core Gameplay

**Puzzle**:
A 9×9 Sudoku starting grid with a guaranteed unique solution and an assigned Difficulty.
_Avoid_: Level, stage, map

**Game**:
A player's single play session on a Puzzle (either a Standard Game or a Daily Challenge), tracking board progress, Mistakes, Hints used, Solve Time, and status (`Active`, `Paused`, `Completed`, or `Abandoned`). At most one Standard Game and one Daily Challenge may be in progress simultaneously.
_Avoid_: Match, session, run

**Daily Challenge**:
A deterministic offline Puzzle seeded by a local calendar date (`YYYY-MM-DD`) with Difficulty assigned by day of the week. Past dates are playable for catch-up, future dates are locked, and each date is recorded once upon completion.
_Avoid_: Daily puzzle, daily mode, event

**Difficulty**:
The skill rating of a Puzzle (`Easy`, `Medium`, `Hard`, or `Expert`), determined by the hardest human solving technique required to complete it without guessing.
_Avoid_: Level, mode, tier

**Given**:
A pre-filled starting digit of a Puzzle that cannot be edited or erased during a Game.
_Avoid_: Clue, fixed cell, initial value

**Notes**:
Candidate digits placed by the player inside an empty cell as scratchpad annotations; placing a correct digit automatically clears that digit from Notes in the same row, column, and 3×3 box.
_Avoid_: Pencil marks, candidates, drafts

**Mistake**:
An incorrect digit placed in an editable cell, determined immediately against the puzzle's unique solution. Mistakes are unlimited and permanent once recorded; erasing, overwriting, or undoing a cell does not remove a Mistake.
_Avoid_: Wrong count, error, strike, violation

**Hint**:
An on-demand solving suggestion that identifies either an existing Mistake to correct or the next logically deducible cell with its solving technique, allowing the player to apply it at a score penalty. Hints are unlimited per game.
_Avoid_: Reveal, clue, cheat, help

### Scoring

**Solve Time**:
Total active elapsed time spent on a Game while unpaused and visible, starting when the Puzzle is shown and ending upon completion.
_Avoid_: Timer, clock, duration

**Earned Score**:
Points accumulated from placing correct digits, scaled by Difficulty. Each editable cell awards points at most once and never when filled via a Hint.
_Avoid_: Raw score, base points

**Time Bonus**:
Additional points awarded upon completing a Puzzle faster than its Difficulty target time, scaling smoothly down to zero.
_Avoid_: Speed bonus

**Penalty**:
Points deducted from a game's score for each Mistake and each Hint used, scaled by Difficulty.
_Avoid_: Minus score, deduction

**Final Score**:
The resulting score for a game, calculated as `Earned Score + Time Bonus − Penalty`, floored at zero.
_Avoid_: Actual score, net score, total score

**Flawless Solve**:
A completed Game finished with zero Mistakes and zero Hints used.
_Avoid_: Perfect game, clean run

**Daily Streak**:
Consecutive calendar days with a completed Daily Challenge leading up to today or yesterday.
_Avoid_: Streak, calendar streak

**Win Streak**:
Consecutive Standard Games completed without abandoning.
_Avoid_: Streak, play streak
