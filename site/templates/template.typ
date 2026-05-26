#let article(meta: dictionary) = body => {
  set document(
    title: meta.at("title", default: none),
    author: meta.at("author", default: none),
    description: meta.at("description", default: none),
    keywords: meta.at("tags", default: none),
  )

  html.link(rel: "stylesheet", href: "../static/style.css")

  html.nav()[
    #html.a(href: "../posts/index.html")[Posts]
    #text(" · ")
    #html.a(href: "../tags/index.html")[Tags]
  ]

  html.main()[
    #html.article()[
      #html.header()[
        #let title = meta.at("title", default: none);
        #if title != none [
          #html.h1()[#title]
        ]

        #let author = meta.at("author", default: ());
        #let desc = meta.at("description", default: none);
        #if author != () or desc != none [
          #html.section()[
            #if author != () [
              #html.address()[
                #{
                  if type(author) == array {
                    author.join(", ")
                  } else {
                    author
                  }
                }
              ]
            ]
            #if desc != none [
              #html.p()[#desc]
            ]
          ]
        ]

        #let tags = meta.at("tags", default: ());
        #if tags != () [
          #html.ul()[
            #{
              if type(tags) == array {
                tags.map(t => html.li()[#html.a(href: "../tags/" + t + ".html")[#t]]).join()
              } else {
                html.li()[#html.a(href: "../tags/" + tags + ".html")[#tags]]
              }
            }
          ]
        ]
      ]

      #body
    ]
  ]
}
