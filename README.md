# gTrends
## Google Trends for Whatever

Google Trends is a powerful tool wich let you watch the people interests in timeline. It is used in marketing researchs, 
advertisment analisys or comparing trend lines with other data. 

gTrends solves the main Google Trends limitation:
  * No API to use it in code
  * Impossibility of comparing more than 5 keywords
  
This project represents a Web API for Google Trends with easy to use functionality. It is not deployed yet, but it is already working on localhost.
To run the webserver just compile this rust project. There is an example wich you could run with this command in your terminal.

`cargo run --example chart`

It will create a chart comparing 6 programming languages.
![Language popularity](/chart.svg)

The work on the project is still in progress. Here is some milestones to do:

- [x] Make the API work
- [x] Simple web site to show how it works
- [ ] Example of comparing Bitcoin price from different source with its trend line
- [x] Quickstart guide
