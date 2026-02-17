with
  company as (
    insert into company (name, default_address_id)
      values ('first', uuidv4()), ('second', uuidv4())
      returning company_id, name, default_address_id
    ),
  contact_value (company_name, name, email, phone_number) as (
    values
      ('first', 'Thierry Wütz', 'thierrywutz@first.fr', null),
      ('second', 'Caroline Pagan', 'c.pagan@second.com', '+33661234567')
    ),
   company_contact as (
    insert into contact (name, email, phone_number, company_id)
      select cv.name, cv.email, cv.phone_number, company.company_id
      from contact_value cv
        join company on company.name = cv.company_name
      returning contact_id, name, email, phone_number, company_id
    ),
  address_value (company_name, label, content, zipcode, city) as (
    values 
      ('first', 'FIRST HQ', '3 rue de la marche', '57300', 'Mouzillon-Sur-Moselle'),
      ('second', 'SECOND_HQ', '1, place du carré vert', '13820', 'Mingon-En-Provence')
    )
insert into address (address_id, label, company_id, content, zipcode, city, associated_contact_id)
  select company.default_address_id, av.label, company.company_id, av.content, av.zipcode, av.city, cc.contact_id
  from company
    join address_value av on company.name = av.company_name
    left join company_contact cc on company.company_id = cc.company_id
