#let article(meta: dictionary) = body => {
  set document(
    title: meta.title,
    author: meta.author,
    description: meta.description,
    keywords: meta.tags,
  )

  html.link(rel: "stylesheet", href: "./style.css")

  html.main()[
    #html.article()[
      #html.header()[
        #if meta.title != none [
          #html.h1()[#meta.title]
        ]

        #if meta.author != () or meta.description != none [
          #html.section()[
            #if meta.author != () [
              #html.address()[
                #if type(meta.author) == array {
                  meta.author.join(", ")
                } else {
                  meta.author
                }
              ]
            ]
            #if meta.description != none [
              #html.p()[#meta.description]
            ]
          ]
        ]

        #if meta.tags != () [
          #html.ul()[
            #(
              if type(meta.tags) == array {
                meta.tags.map(t => html.li()[#t]).join()
              } else {
                html.li()[#meta.tags]
              }
            )
          ]
        ]
      ]

      #body
    ]
  ]
}
