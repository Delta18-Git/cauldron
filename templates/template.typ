#let article(
  title: none,
  author: (),
  description: none,
  tags: (),
) = body => {
  set document(
    title: title,
    author: author,
    description: description,
    keywords: tags,
  )

  html.link(rel: "stylesheet", href: "./style.css")

  html.main()[
    #html.article()[
      #html.header()[
        #if title != none [
          #html.h1()[#title]
        ]

        #if author != () or description != none [
          #html.section()[
            #if author != () [
              #html.address()[
                #if type(author) == array {
                  author.join(", ")
                } else {
                  author
                }
              ]
            ]
            #if description != none [
              #html.p()[#description]
            ]
          ]
        ]

        #if tags != () [
          #html.ul()[
            #(
              if type(tags) == array {
                tags.map(t => html.li()[#t]).join()
              } else {
                html.li()[#tags]
              }
            )
          ]
        ]
      ]

      #body
    ]
  ]
}
