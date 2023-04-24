# Agrum

Agrum is a crate designed to make the SQL code maintainable and testable while letting developpers to focus on the business value of the queries.

This library is still in early development stage, means it is **not production ready**.

## What is Agrum?

Agrum organizes the database code in 3 layers:

 * a business oriented layer that defines what queries need to be performs and deals with conditions and parameters (clear code easy to modify).
 * a query layer that is separated into projections and definitions that can be unit tested (not often modified)
 * a database layer that hydrate Rust structures from SQL results (low level, isolated)

## How to work with Agrum?

Determine what SQL query you want to issue using your favorite SQL client (not to say `psql`). Once you know exactly the SQL query you need, put it as a test for the `SqlDefinition` you will create. This SQL definition will be split in two responsabilities:
 * the projection that is required to build the Rust structure that will hold the data (the `SELECT` part)
 * the conditions that can vary using the same SQL query to represent different data contexts (the `WHERE` part)

The same query can be used to hydrate several kinds of Rust structures. The same query for the same structure can be used with different set of conditions.

