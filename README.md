## RUST-Based programming language for CV creation.


This project presents a very simplified programming language for CV creation based-off RUST and LaTeX. 

For now, the CVs are based off Jake's resume template.

#### Some examples:

```
name "Youssef Briki"
contact email "youssef@example.com", github "youssefbriki1", location "Montréal, QC"

section "Experience":
  entry role "AI Engineering Intern"
        org  "Desjardins"
        when "Summer 2025"
        bullets:
          - "Built domain-specific RAG on a knowledge graph"
          - "Reduced retrieval latency by 35% (P95)"
          - "Productionized pipelines on Balam HPC cluster"

section "Projects":
  entry role "LabMate – Lab Q&A Agent"
        org  "Personal / AC collab"
        when "2024–2025"
        bullets:
          - "Multi-hop retrieval over manuals; vLLM serving"
          - "Tools: FAISS, LangChain, spaCy coref, SRL microservices"

section "Education":
  entry role "B.Sc. Computer Science"
        org  "Université de Montréal"
        when "2022–2026"
        bullets:
          - "GPA 3.7/4.3, ICPC participant"
          - "NLP, ML, HPC coursework"

section "Skills":
  tags: "Python, Rust, Java, LangChain, vLLM, FAISS, Docker, gRPC, HPC"
```

Or 

```
# New constructs (suggested)
summary:
  - "SWE + NLP, focused on RAG and knowledge graphs for automation."

section "Experience":
  entry role "AI Engineering Intern"
        org  "Desjardins"
        when "Summer 2025"
        location "Montréal, QC"
        link "https://desjardins.com"
        stack: "Python, LangChain, FAISS, vLLM, Docker, GCP"
        bullets:
          - "Built domain RAG over Desjardins knowledge graph"
          - "Cut retrieval P95 by 35% via cache + re-ranking"

section "Projects":
  entry role "LabMate – Lab Q&A Agent"
        org  "Acceleration Consortium"
        when "2024–2025"
        link "https://github.com/your/labmate"
        stack: "Python, vLLM, spaCy-coref, SRL, gRPC, FAISS"
        bullets:
          - "Procedural KG extraction and multi-hop retrieval"
          - "gRPC microservices for SRL/NER/coref; Dockerized"

# Jake-style sidebar (rendered in left column by your template)
sidebar:
  location "Montréal, QC"
  email "youssef@example.com"
  github "github.com/youssefbriki1"
  linkedin "linkedin.com/in/youssefbriki"
  languages "English, French"
  skills "Python, Rust, Java, RAG, KG, vLLM, FAISS, Docker"




```

Or 


```
name "Youssef Briki"
contact email "youssef@example.com", github "youssefbriki1"

summary:
  - "Undergrad CS (grad 2026). Passion: NLP + HPC; building reliable RAG."

section "Experience":
  entry role "NLP Intern"
        org  "AC / UofT"
        when "Winter 2025"
        stack: "Python, spaCy, SRL, FAISS"
        bullets:
          - "Authored SRL microservice used by 2 internal teams"
          - "Cut preprocessing time 40% by batching + caching"

section "Projects":
  entry role "KG-Builder"
        org  "Course + personal"
        when "2024"
        link "https://github.com/you/kg-builder"
        stack: "RDF, NetworkX, PostgreSQL"
        bullets:
          - "Turned papers into procedural graphs; query with Cypher-like DSL"

section "Education":
  entry role "B.Sc. Computer Science"
        org  "UdeM"
        when "2022–2026"
        bullets:
          - "GPA 3.7/4.3; relevant: NLP, ML, Databases, Parallel Prog"

section "Skills":
  tags: "Python, Rust, Java, SQL, Docker, Git, Linux"
```
