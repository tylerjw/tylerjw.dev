+++
title = "Fearless Git Collaboration"
date = 2024-10-14
type = "post"
in_search_index = true
[taxonomies]
tags = ["Software"]
+++

Good art sells terrible ideas.

Don't believe me?
Do you remember [GitFlow](https://leanpub.com/git-flow/read).
That website is a fly-trap for teams led by inexperienced software engineers wanting to be told how to use Git.
If you have been around for a while, you likely have been on a team that fell victim to it if it wasn't you who brought it to your team.

Why do we do we inflict things like GitFlow on ourselves?
It has to be more than just those colorful diagrams.

It is more, but I must tell you about my current project.
The tools we get with Rust are killer, and one of those tools is the packaging system.

When this project started, we needed to set up a handful of relatively unrelated applications to fulfill different roles in a micro-services system.
To move quickly, we chose the default path in Rust and set up a separate repo for each tool to build one or two of them.
This helped us move quickly; tests ran fast, and CI was fast.

But as our project grew in complexity, various things got factored into libraries, which got their own repos.
We automated packaging on each commit into our private package repository so we could easily depend on each other.
Then came the project of how we roll out updates to dependencies, and we wrote a bot.

That bot helpfully goes around creating patches for every repo where a dependency has updated, and if there are code changes needed, CI fails, so we know we need to do more work.
At this point, you probably think a hundred of us are working on this project with hundreds of thousands of lines of code.

That couldn't be further from the truth.
We are 5 people with 18 repos and a wild amount of infrastructure to propagate changes from one developer to another.
Good luck making a change in a handful of repos at once and landing that change on the same day.
It is just wrong that it takes multiple days for a team of five developers to land a change across the whole project.

And now we are confronted with what do we do about this mess.
You can bet I'm pitching to reduce the system's complexity by merging everything into a single repo so we can collaborate in Git.
But the biggest fear in the team is that if we do that, we will conflict with our changes, which will slow us down.
Ultimately, it comes down to the unsaid opinion of programmers that Git is not a good tool for collaboration.
While excellent, the Rust packaging system is not a collaboration tool, but we use it like it is.

The reason GitFlow worked was not just the colorful diagrams; it was the subtle message that it would help solve the problem many programmers believe.
As a group, we intuit that Git is not a tool for collaboration.
How can we change that perception?

What advice do you have for convincing my team that Git is the right tool for us to collaborate over code and build something together?
